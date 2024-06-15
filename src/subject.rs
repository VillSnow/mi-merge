use std::{future::Future, pin::Pin};

use tokio::sync::Mutex;

pub struct SubjectMut<T>
where
    T: Clone + Send,
{
    callbacks: Mutex<Vec<Box<dyn FnMut(T) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>>>,
}

impl<T> SubjectMut<T>
where
    T: Clone + Send,
{
    pub fn new() -> Self {
        Self {
            callbacks: Mutex::new(Vec::new()),
        }
    }

    pub async fn next(&self, value: T) {
        let mut callbacks = self.callbacks.lock().await;
        for f in callbacks.iter_mut() {
            f(value.clone()).await;
        }
    }

    pub async fn subscribe<F, G>(&mut self, mut callback: F)
    where
        F: (FnMut(T) -> G) + Send + 'static,
        G: Future<Output = ()> + Send + 'static,
    {
        let mut callbacks = self.callbacks.lock().await;
        callbacks.push(Box::new(move |m| Box::pin(callback(m))));
    }
}

impl<T> Default for SubjectMut<T>
where
    T: Clone + Send,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> std::fmt::Debug for SubjectMut<T>
where
    T: Clone + Send,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubjectMut").finish()
    }
}
