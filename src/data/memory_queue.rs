use std::{collections::LinkedList, error::Error};

use async_trait::async_trait;

use crate::core::{errors::WebMQError, traits::AsyncQueue};

pub struct MemoryQueue<T> {
    queue: LinkedList<T>
}

#[async_trait]
impl<T: Send + Sync> AsyncQueue<T> for MemoryQueue<T> {
    async fn pop(&mut self) -> Result<T, Box<dyn Error>> {
        let Some(data) = self.queue.pop_front() else {
            return Err(Box::new(WebMQError::Data("Failed to pop data from queue as it contains no elements".into())))
        };

        Ok(data)
    }

    async fn push(&mut self, data: T) -> Option<Box<dyn Error>> {
        self.queue.push_back(data);
        None
    }
}

impl<T> MemoryQueue<T> {
    pub fn new() -> MemoryQueue<T> {
        MemoryQueue {
            queue: LinkedList::new()
        }
    }
}