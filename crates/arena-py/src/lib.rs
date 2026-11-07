//! Python module `arena._arena`: a batch of fights that Python steps with one call.
//!
//! The package `arena` in `python/` re-exports it, and `_arena.pyi` there holds the type
//! hints. Keep both the same as this file.

use arena_env::{
    decode::{ACTION_DIMS, Choice},
    observe::OBS_LEN,
    reward::Weights,
};
use numpy::{PyArray1, PyArray2, PyArrayMethods, PyReadonlyArray2, ndarray::ArrayView1};
use pyo3::{exceptions::PyValueError, prelude::*};

/// Fights that step together. A fight that ends starts again in the same step.
#[pyclass]
struct VecEnv(arena_env::vec_env::VecEnv);

#[pymethods]
impl VecEnv {
    #[new]
    #[pyo3(signature = (
        num_envs,
        seed = 0,
        frame_skip = 4,
        hit_dealt = 1.0,
        hit_taken = -1.0,
        win = 10.0,
        loss = -10.0,
    ))]
    fn new(
        num_envs: usize,
        seed: u64,
        frame_skip: u32,
        hit_dealt: f32,
        hit_taken: f32,
        win: f32,
        loss: f32,
    ) -> PyResult<Self> {
        if num_envs == 0 || frame_skip == 0 {
            return Err(PyValueError::new_err(
                "num_envs and frame_skip must be at least 1",
            ));
        }
        let weights = Weights::new(hit_dealt, hit_taken, win, loss);
        Ok(Self(arena_env::vec_env::VecEnv::new(
            num_envs, weights, frame_skip, seed,
        )))
    }

    #[getter]
    fn num_envs(&self) -> usize {
        self.0.num_envs()
    }

    /// Start a new fight in every env. Return the observations, shape `(num_envs, OBS_LEN)`.
    fn reset<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f32>>> {
        let n = self.0.num_envs();
        let mut obs = vec![0.0; n * OBS_LEN];
        self.0.reset(&mut obs);
        PyArray1::from_vec(py, obs).reshape([n, OBS_LEN])
    }

    /// Step every fight once. `actions` has shape `(num_envs, len(ACTION_DIMS))`.
    ///
    /// Return the observations, the rewards and the done flags. The observation of a done
    /// fight is the first one of its next fight.
    #[expect(clippy::type_complexity, reason = "the tuple that Python unpacks")]
    #[expect(
        clippy::needless_pass_by_value,
        reason = "PyO3 passes arguments by value"
    )]
    fn step<'py>(
        &mut self,
        py: Python<'py>,
        actions: PyReadonlyArray2<'py, i64>,
    ) -> PyResult<(
        Bound<'py, PyArray2<f32>>,
        Bound<'py, PyArray1<f32>>,
        Bound<'py, PyArray1<bool>>,
    )> {
        let actions = actions.as_array();
        let n = self.0.num_envs();
        if actions.shape() != [n, ACTION_DIMS.len()] {
            return Err(PyValueError::new_err(format!(
                "actions must have shape ({n}, {}), not {:?}",
                ACTION_DIMS.len(),
                actions.shape()
            )));
        }
        let mut choices = Vec::with_capacity(n);
        for row in actions.rows() {
            choices.push(choice(&row)?);
        }

        let mut obs = vec![0.0; n * OBS_LEN];
        let mut rewards = vec![0.0; n];
        let mut dones = vec![false; n];
        self.0.step(&choices, &mut obs, &mut rewards, &mut dones);
        Ok((
            PyArray1::from_vec(py, obs).reshape([n, OBS_LEN])?,
            PyArray1::from_vec(py, rewards),
            PyArray1::from_vec(py, dones),
        ))
    }
}

/// Return the choice in one row of actions, or an error for a value out of range.
fn choice(row: &ArrayView1<i64>) -> PyResult<Choice> {
    let mut choice = Choice::default();
    for ((slot, &value), &dim) in choice.iter_mut().zip(row).zip(&ACTION_DIMS) {
        *slot = usize::try_from(value)
            .ok()
            .filter(|&v| v < dim)
            .ok_or_else(|| PyValueError::new_err(format!("action {value} is not in 0..{dim}")))?;
    }
    Ok(choice)
}

#[pymodule]
#[pyo3(name = "_arena")]
fn arena(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<VecEnv>()?;
    m.add("OBS_LEN", OBS_LEN)?;
    m.add("ACTION_DIMS", ACTION_DIMS.to_vec())?;
    Ok(())
}
