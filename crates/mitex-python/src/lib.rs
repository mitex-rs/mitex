use pyo3::prelude::*;
use mitex::convert_math; // Your existing converter function

#[pyfunction]
fn convert_latex_to_typst(text: &str) -> PyResult<String> {
    match convert_math(text, None) {
        Ok(result) => Ok(result),
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }
}

#[pymodule]
fn mitex_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(convert_latex_to_typst, m)?)?;
    Ok(())
}