use std::{collections::HashMap, pin::Pin};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::core::{errors::WebMQError, traits::AsyncQueue};

use crate::core::traits::MessagingDispatcher;

type QueueFac = Pin<Box<dyn Fn() -> Box<dyn AsyncQueue<Vec<u8>> + Send> + Send + Sync>>;

pub struct BaseMessagingDispatcher {
    queues: Mutex<HashMap<String, Box<dyn AsyncQueue<Vec<u8>> + Send>>>,
    queue_factory: QueueFac
}

#[async_trait]
impl MessagingDispatcher<String, Vec<u8>> for BaseMessagingDispatcher {
    async fn consume(&mut self, queue: String) -> Result<Vec<u8>, WebMQError> {
        let mut queues = self.queues.lock().await;

        if let Some(mut_queue) = queues.get_mut(queue.as_str()) {
            match mut_queue.pop().await {
                Ok(r) => Ok(r),
                Err(e) => Err(WebMQError::Data(e.to_string()))
            }
        } else {
            Err(WebMQError::Data(format!("No messages in queue {queue}")))
        }
    }

    async fn publish(&mut self, queue: String, data: Vec<u8>) -> Option<WebMQError> {
        let mut queues = self.queues.lock().await;

        if !queues.contains_key(&queue) {
            queues.insert(queue.clone(), self.queue_factory.as_ref()());
        }

        if let Some(mut_queue) = queues.get_mut(&queue) {
            mut_queue.push(data).await;
            None
        } else {
            Some(WebMQError::Data(format!("Could not publish to queue {queue}.")))
        }
    }
}

impl BaseMessagingDispatcher
{
    pub fn new(queue_factory: QueueFac) -> BaseMessagingDispatcher {
        BaseMessagingDispatcher {
            queues: HashMap::new().into(),
            queue_factory
        }
    }
}