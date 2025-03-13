mod output;

use aws_sdk_qbusiness::operation::chat::builders::ChatFluentBuilder;
use aws_sdk_qbusiness::types::error::ChatInputStreamError;
use aws_sdk_qbusiness::types::{ChatInputStream, EndOfInputEvent, TextInputEvent};
use aws_smithy_http::event_stream::EventStreamSender;
use futures_util::{Stream, StreamExt};
use output::ChatOutputGenerator;
use pyo3::prelude::*;

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
    client: aws_sdk_qbusiness::Client,
}

#[pymethods]
impl QBusiness {
    #[new]
    fn new() -> PyResult<Self> {
        // Create a Tokio runtime to load AWS configuration and create the SDK client
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Failed to create Tokio runtime: {:?}",
                e
            ))
        })?;
        let client = rt.block_on(async {
            let config = aws_config::load_from_env().await;
            aws_sdk_qbusiness::Client::new(&config)
        });
        Ok(QBusiness { client })
        // Ok(QBusiness {})
    }

    /// Prepares a chat session for asynchronous chat operations.
    /// Takes an account ID, application ID, and an optional user ID.
    #[pyo3(signature = (application_id, user_id=None))]
    fn prepare_chat(
        &self,
        application_id: String,
        user_id: Option<String>,
    ) -> PyResult<ChatSession> {
        // For now, just create a new ChatSession with the provided values.
        Ok(ChatSession {
            application_id,
            user_id,
            client: self.client.clone(),
        })
    }
}

#[pyclass]
struct ChatSession {
    application_id: String,
    user_id: Option<String>,
    client: aws_sdk_qbusiness::Client,
}

#[pymethods]
impl ChatSession {
    /// Outline for the send_chat method.
    /// This method will eventually accept a Python async iterable of chat input events
    /// and return an async iterable of chat output events.
    fn send_chat<'p>(
        &mut self,
        py: Python<'p>,
        input_events: Bound<PyAny>,
    ) -> PyResult<Bound<'p, PyAny>> {
        let input_events = stream_input_events(input_events)?;
        let app = std::mem::replace(&mut self.application_id, String::new());
        let user_id = self.user_id.take();
        let client = self.client.to_owned();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let sender = EventStreamSender::from(input_events);
            let chat_builder = client
                .chat()
                .application_id(app)
                .set_user_id(user_id)
                .input_stream(sender);
            let out_stream = call_and_get_stream(chat_builder).await?;
            Ok(out_stream)
        })
    }
}

async fn call_and_get_stream(builder: ChatFluentBuilder) -> PyResult<ChatOutputGenerator> {
    let chat_output = builder.send().await;
    let chat_output = match chat_output {
        Err(e) => {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                e.into_service_error().to_string(),
            ))
        }
        Ok(output) => output,
    };
    let stream_receiver = chat_output.output_stream;
    Ok(ChatOutputGenerator::new(stream_receiver))
}

fn stream_input_events(
    input_events: Bound<PyAny>,
) -> PyResult<impl Stream<Item = Result<ChatInputStream, ChatInputStreamError>>> {
    let stream = pyo3_async_runtimes::tokio::into_stream_v2(input_events)?
        .map(|item| {
            Python::with_gil(|py| -> PyResult<ChatInputStream> {
                let obj = item.bind(py);
                let event = convert_chat_input_event(obj)?;
                Ok(event)
            })
        })
        .map(|input| match input {
            Ok(event) => Ok(event),
            Err(err) => Err(ChatInputStreamError::unhandled(err)),
        });
    Ok(stream)
}

fn convert_chat_input_event(raw: &Bound<PyAny>) -> PyResult<ChatInputStream> {
    let type_str: String = raw.get_item("type")?.extract()?;

    // Is there a better way to do this?
    let event = match type_str.as_str() {
        "text" => {
            let user_message: String = raw.get_item("user_message")?.extract()?;
            let text_event = TextInputEvent::builder()
                .user_message(user_message)
                .build()
                .map_err(|e| {
                    pyo3::exceptions::PyValueError::new_err(format!(
                        "Failed to build TextInputEvent: {:?}",
                        e
                    ))
                })?;
            ChatInputStream::TextEvent(text_event)
        }
        "end" => ChatInputStream::EndOfInputEvent(EndOfInputEvent::builder().build()),
        _ => {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Unknown input type: {}",
                type_str
            )));
        }
    };

    Ok(event)
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
