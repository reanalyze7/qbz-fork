use super::*;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

#[tokio::test]
async fn succeeds_first_try() {
    let calls = Arc::new(AtomicU32::new(0));
    let c = calls.clone();
    let r: Result<u32, FetchError> = retry_transient(3, "test", FetchError::is_transient, |_| {
        let c = c.clone();
        async move {
            c.fetch_add(1, Ordering::Relaxed);
            Ok(42)
        }
    })
    .await;
    assert_eq!(r.unwrap(), 42);
    assert_eq!(calls.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn retries_transient_then_succeeds() {
    let calls = Arc::new(AtomicU32::new(0));
    let c = calls.clone();
    let r: Result<u32, FetchError> = retry_transient(3, "test", FetchError::is_transient, |attempt| {
        let c = c.clone();
        async move {
            c.fetch_add(1, Ordering::Relaxed);
            if attempt < 3 {
                Err(FetchError::Transient("503".into()))
            } else {
                Ok(7)
            }
        }
    })
    .await;
    assert_eq!(r.unwrap(), 7);
    assert_eq!(calls.load(Ordering::Relaxed), 3);
}

#[tokio::test]
async fn terminal_does_not_retry() {
    let calls = Arc::new(AtomicU32::new(0));
    let c = calls.clone();
    let r: Result<u32, FetchError> = retry_transient(3, "test", FetchError::is_transient, |_| {
        let c = c.clone();
        async move {
            c.fetch_add(1, Ordering::Relaxed);
            Err(FetchError::Terminal("404".into()))
        }
    })
    .await;
    assert!(r.is_err());
    assert_eq!(calls.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn gives_up_after_max_attempts() {
    let calls = Arc::new(AtomicU32::new(0));
    let c = calls.clone();
    let r: Result<u32, FetchError> = retry_transient(3, "test", FetchError::is_transient, |_| {
        let c = c.clone();
        async move {
            c.fetch_add(1, Ordering::Relaxed);
            Err(FetchError::Transient("timeout".into()))
        }
    })
    .await;
    assert!(r.is_err());
    assert_eq!(calls.load(Ordering::Relaxed), 3);
}
