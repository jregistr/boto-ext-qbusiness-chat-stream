use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pyfunction]
fn hello_from_rust() -> PyResult<String> {
    Ok("Hello from Rust!".to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn pyo3_q_stream(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(hello_from_rust, m)?)?;
    Ok(())
}
