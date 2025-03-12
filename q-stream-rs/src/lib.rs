use aws_sdk_qbusiness::types::{ChatInputStream, TextInputEvent};
use futures_util::{StreamExt, TryStreamExt};
use pyo3::prelude::*;
use pyo3::types::PyString;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pyfunction]
fn say_hello() -> PyResult<String> {
    Ok("hello big world!!".to_string())
}

#[pyfunction]
fn say_it_out_loud() -> PyResult<String> {
    Ok("hello out loud world!!".to_string())
}

#[pyclass]
struct QBusiness {
    // client: aws_sdk_qbusiness::Client,
}

#[pymethods]
impl QBusiness {
    #[new]
    fn new() -> PyResult<Self> {
        // Create a Tokio runtime to load AWS configuration and create the SDK client
        /*  let rt = tokio::runtime::Runtime::new().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Failed to create Tokio runtime: {:?}",
                e
            ))
        })?;
        let client = rt.block_on(async {
            let config = aws_config::load_from_env().await;
            aws_sdk_qbusiness::Client::new(&config)
        });
        Ok(QBusiness { client })*/
        Ok(QBusiness {})
    }

    /// Prepares a chat session for asynchronous chat operations.
    /// Takes an account ID, application ID, and an optional user ID.
    fn prepare_chat(
        &self,
        account_id: String,
        application_id: String,
        user_id: Option<String>,
    ) -> PyResult<ChatSession> {
        // For now, just create a new ChatSession with the provided values.
        Ok(ChatSession {
            account_id,
            application_id,
            user_id,
            // client: self.client.clone(),
        })
    }
}

#[pyclass]
struct ChatSession {
    account_id: String,
    application_id: String,
    user_id: Option<String>,
    // client: aws_sdk_qbusiness::Client,
}

struct AA {}

#[pymethods]
impl ChatSession {
    /// Outline for the send_chat method.
    /// This method will eventually accept a Python async iterable of chat input events
    /// and return an async iterable of chat output events.
    fn send_chat<'p>(&self, py: Python<'p>, py_input: Bound<PyAny>) -> PyResult<Bound<'p, PyAny>> {
        use async_stream::stream;
        use futures_util::StreamExt;

        // .map(|item| Python::with_gil(|py| -> PyResult<i32> { Ok(item.bind(py).extract()?) }))
        let py_stream = pyo3_async_runtimes::tokio::into_stream_v2(py_input)?.map(|item| {
            Python::with_gil(|py| -> PyResult<ChatInputStream> {
                let obj = item.bind(py);
                let event = convert_chat_input_event(obj)?;
                Ok(event)
                // Ok(item.bind(py).extract()?)
            })
        });

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let collected = py_stream.try_collect::<Vec<_>>().await;

            println!("collected: {:?}", collected);
            Ok(())
        })

        /*    .map(|item| {
            let gil = Python::with_gil(|py| -> PyResult<i32>  {
                item.bind(py).extract()?
            });

        });*/

        // Ok(&*PyString::new(py, "placeholder string"))
        // let a = "hello there, finished".to_string();
        // Ok(a)
        // todo!()
        // Placeholder implementation
        // Ok("send_chat not implemented yet".to_string())
    }
}

fn convert_chat_input_event(raw_input_event: &Bound<PyAny>) -> PyResult<ChatInputStream> {
    println!("raw input event: {:?}", raw_input_event);
    Ok(ChatInputStream::TextEvent(
        TextInputEvent::builder()
            .user_message("hello")
            .build()
            .unwrap(),
    ))
}

/// A Python module implemented in Rust.
#[pymodule]
fn q_stream_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(say_hello, m)?)?;
    m.add_function(wrap_pyfunction!(say_it_out_loud, m)?)?;
    m.add_class::<QBusiness>()?;
    Ok(())
}
