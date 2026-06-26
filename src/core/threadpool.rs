use std::sync::mpsc;
use std::time::Duration;
use threadpool::ThreadPool;

static POOL: once_cell::sync::OnceCell<ThreadPool> = once_cell::sync::OnceCell::new();

fn get_pool() -> &'static ThreadPool {
    POOL.get_or_init(|| {
        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        ThreadPool::new(num_threads)
    })
}

#[derive(Debug)]
pub enum PoolError {
    Timeout { timeout_secs: u64 },
    Execute(String),
}

impl std::fmt::Display for PoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PoolError::Timeout { timeout_secs } => {
                write!(f, "Operation timed out after {}s", timeout_secs)
            }
            PoolError::Execute(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for PoolError {}

pub fn with_timeout<F, T>(f: F, timeout_secs: u64) -> Result<T, PoolError>
where
    F: FnOnce() -> Result<T, anyhow::Error> + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = mpsc::channel();

    get_pool().execute(move || {
        let result = f();
        let _ = tx.send(result);
    });

    match rx.recv_timeout(Duration::from_secs(timeout_secs)) {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(e)) => Err(PoolError::Execute(e.to_string())),
        Err(_) => Err(PoolError::Timeout { timeout_secs }),
    }
}
