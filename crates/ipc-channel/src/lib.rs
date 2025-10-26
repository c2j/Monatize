use std::marker::PhantomData;

use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;
use tokio::sync::mpsc;

#[derive(Debug, Error)]
pub enum SendError {
    #[error("channel closed")]
    Closed,
    #[error("encode error: {0}")]
    Encode(String),
}

#[derive(Debug, Error)]
pub enum RecvError {
    #[error("channel closed")]
    Disconnected,
    #[error("decode error: {0}")]
    Decode(String),
}

/// A typed sender that serializes messages with bincode
#[derive(Debug, Clone)]
pub struct Sender<T> {
    inner: mpsc::Sender<Vec<u8>>, // serialized bytes
    _pd: PhantomData<T>,
}

/// A typed receiver that deserializes messages with bincode
#[derive(Debug)]
pub struct Receiver<T> {
    inner: mpsc::Receiver<Vec<u8>>, // serialized bytes
    _pd: PhantomData<T>,
}

impl<T> Sender<T>
where
    T: Serialize + Send + 'static,
{
    pub async fn send(&self, value: T) -> Result<(), SendError> {
        let bytes = bincode::serialize(&value).map_err(|e| SendError::Encode(e.to_string()))?;
        self.inner
            .send(bytes)
            .await
            .map_err(|_| SendError::Closed)
    }
}

impl<T> Receiver<T>
where
    T: DeserializeOwned + Send + 'static,
{
    pub async fn recv(&mut self) -> Result<T, RecvError> {
        match self.inner.recv().await {
            Some(bytes) => bincode::deserialize::<T>(&bytes).map_err(|e| RecvError::Decode(e.to_string())),
            None => Err(RecvError::Disconnected),
        }
    }
}

/// Create an "unbounded" in-process channel pair (implemented as a large bounded channel)
pub fn unbound<T>() -> (Sender<T>, Receiver<T>)
where
    T: Serialize + DeserializeOwned + Send + 'static,
{
    // Tokio's unbounded channel uses a different type; for simplicity we use a large bounded channel here.
    let (tx, rx) = mpsc::channel::<Vec<u8>>(1024);
    (
        Sender { inner: tx, _pd: PhantomData },
        Receiver { inner: rx, _pd: PhantomData },
    )
}

/// Create a bounded in-process channel pair
pub fn bounded<T>(cap: usize) -> (Sender<T>, Receiver<T>)
where
    T: Serialize + DeserializeOwned + Send + 'static,
{
    let (tx, rx) = mpsc::channel::<Vec<u8>>(cap);
    (
        Sender { inner: tx, _pd: PhantomData },
        Receiver { inner: rx, _pd: PhantomData },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn ping_pong_latency_basic() {
        let (tx_ab, mut rx_ab) = bounded::<u64>(128);
        let (tx_ba, mut rx_ba) = bounded::<u64>(128);

        let echo = tokio::spawn(async move {
            while let Ok(v) = rx_ab.recv().await {
                let _ = tx_ba.send(v).await;
            }
        });

        let start = std::time::Instant::now();
        let rounds = 100u64;
        for i in 0..rounds {
            tx_ab.send(i).await.unwrap();
            let _ = rx_ba.recv().await.unwrap();
        }
        let dur = start.elapsed();
        // Avoid flakiness in CI: assert per-round avg < 2ms
        let per = dur / (rounds as u32);
        assert!(per < Duration::from_millis(2), "avg per message: {:?}", per);

        drop(tx_ab);
        let _ = echo.await;
    }

    #[tokio::test]
    async fn large_message_1mib() {
        let (tx, mut rx) = bounded::<Vec<u8>>(8);
        let data = vec![42u8; 1024 * 1024];
        tx.send(data.clone()).await.unwrap();
        let got = rx.recv().await.unwrap();
        assert_eq!(got.len(), data.len());
        assert_eq!(got[0], 42);
    }

    #[tokio::test]
    async fn disconnect_error() {
        let (tx, mut rx) = bounded::<u32>(1);
        drop(tx);
        let err = rx.recv().await.err().expect("should be error");
        matches!(err, RecvError::Disconnected);
    }
}

