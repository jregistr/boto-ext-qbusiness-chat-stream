use aws_sdk_qbusiness::primitives::event_stream::EventReceiver;
use aws_sdk_qbusiness::types::error::ChatOutputStreamError;
use aws_sdk_qbusiness::types::ChatOutputStream;
use pyo3::{pyclass, pymethods, PyObject, PyRef, PyResult, Python};
use std::sync::Arc;
use tokio::sync::Mutex;

type ChatEventReceiver = EventReceiver<ChatOutputStream, ChatOutputStreamError>;

#[pyclass]
pub struct ChatOutputGenerator {
    inner: Arc<Mutex<ChatEventReceiver>>,
}

impl ChatOutputGenerator {
    pub fn new(receiver: ChatEventReceiver) -> Self {
        Self {
            inner: Arc::new(Mutex::new(receiver)),
        }
    }
}

#[pyclass]
#[allow(dead_code)]
struct Output {
    #[pyo3(get)]
    kind: String,
    #[pyo3(get)]
    text: Option<String>,
    chat_id: Option<String>,
    user_msg_id: Option<String>,
    sys_msg_id: Option<String>,
}

impl Output {
    fn text(value: String) -> Self {
        Self {
            text: Some(value),
            chat_id: None,
            sys_msg_id: None,
            user_msg_id: None,
            kind: "text".to_string(),
        }
    }

    fn metadata(chat_id: String, user_msg: String, sys_msg: String) -> Self {
        Self {
            text: None,
            chat_id: Some(chat_id),
            user_msg_id: Some(user_msg),
            sys_msg_id: Some(sys_msg),
            kind: "metadata".to_string(),
        }
    }
}

#[pymethods]
impl ChatOutputGenerator {
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__<'a>(&self, py: Python<'a>) -> PyResult<Option<PyObject>> {
        let receiver = self.inner.clone();

        let future = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let next_event = receiver.lock().await.recv().await;
            let next_event = match next_event {
                Ok(n) => n,
                Err(e) => {
                    let err_msg = e.to_string();
                    return Err(pyo3::exceptions::PyException::new_err(err_msg));
                }
            };

            let Some(next_event) = next_event else {
                return Err(pyo3::exceptions::PyStopAsyncIteration::new_err(
                    "Iterator exhausted",
                ));
            };

            let res = match next_event {
                ChatOutputStream::TextEvent(text) => {
                    Output::text(text.system_message.unwrap_or("".to_string()))
                }
                ChatOutputStream::MetadataEvent(metadata) => Output::metadata(
                    metadata.conversation_id.unwrap(),
                    metadata.user_message_id.unwrap(),
                    metadata.system_message_id.unwrap(),
                ),
                _ => Output::text("".to_string()),
            };
            Ok(Some(res))
        });

        let result = future?;
        Ok(Some(result.into()))
    }
}
