use anyhow::Result;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Semaphore;

/// Batch processing constants, consistent with Go version
const N_GOROUTINE: usize = 100;

/// Batch processing function, corresponding to Go version's GoBatch
pub async fn go_batch<F, Fut>(len_batch: usize, f_go: F) -> Result<i64>
where
    F: Fn(usize) -> Fut + Send + Sync + Clone + 'static,
    Fut: std::future::Future<Output = Result<()>> + Send + 'static,
{
    if len_batch == 0 {
        return Ok(0);
    }

    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    let n_batch = (len_batch as f64 / N_GOROUTINE as f64).ceil() as usize;
    let semaphore = Arc::new(Semaphore::new(N_GOROUTINE));

    let mut handles = Vec::new();

    for i in 0..N_GOROUTINE {
        let semaphore = Arc::clone(&semaphore);
        let f_go = f_go.clone();

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            for j in (i * n_batch)..((i + 1) * n_batch) {
                if j >= len_batch {
                    break;
                }

                if let Err(e) = f_go(j).await {
                    return Err(e);
                }
            }

            Ok::<(), anyhow::Error>(())
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await??;
    }

    let end_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    Ok(end_time - start_time)
}
