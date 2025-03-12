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
            pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to create Tokio runtime: {:?}", e))
        })?;
        let client = rt.block_on(async {
            let config = aws_config::load_from_env().await;
            aws_sdk_qbusiness::Client::new(&config)
        });
        Ok(QBusiness { client })
    }

    /// Calls the AWS QBusiness chat method with the given input.
    /// It sends a text event and an end-of-input event, then collects all text and metadata events from the response.
    fn chat(&self, input: String) -> PyResult<String> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to create Tokio runtime: {:?}", e))
        })?;
        let result = rt.block_on(async {
            // Create chat input events: a text event and an end-of-input event.
            let text_event = aws_sdk_qbusiness::model::ChatInputEvent::builder()
                .event_type("text")
                .content(input)
                .build();
            let end_event = aws_sdk_qbusiness::model::ChatInputEvent::builder()
                .event_type("end")
                .build();
            let events = futures::stream::iter(vec![text_event, end_event]);

            // Call the AWS QBusiness chat method which returns a stream of response events
            let mut resp_stream = self.client.chat(events).await.map_err(|e| format!("Chat call failed: {:?}", e))?;

            let mut collected_text = String::new();
            use futures::StreamExt;
            while let Some(event_result) = resp_stream.next().await {
                let event = event_result.map_err(|e| format!("Error reading chat response event: {:?}", e))?;
                if event.event_type() == "text" {
                    collected_text.push_str(&event.text().unwrap_or_default());
                } else if event.event_type() == "metadata" {
                    collected_text.push_str(&format!("[metadata: {}]", event.metadata().unwrap_or_default()));
                }
            }
            Ok(collected_text)
        });

        match result {
            Ok(text) => Ok(text),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!("AWS QBusiness chat error: {}", e))),
        }
    }
}


/// A Python module implemented in Rust.
#[pymodule]
fn q_stream_rs(m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(say_hello, m)?)?;
    m.add_class::<QBusiness>()?;
    Ok(())
}
