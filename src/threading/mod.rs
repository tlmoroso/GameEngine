use std::sync::{RwLock, Arc};
use rayon::{ThreadPool, ThreadPoolBuildError};
use thiserror::Error;

pub struct Pool(pub Arc<RwLock<rayon::ThreadPool>>);

impl Pool {
    pub fn new(pool: ThreadPool) -> Self {
        Self {
            0: Arc::new(RwLock::new(pool))
        }
    }
}

#[derive(Error, Debug)]
pub enum ThreadError {
    #[error("Failed to acquire read lock from Pool")]
    PoolReadLockError,
    #[error("Failed to build rayon thread pool")]
    ThreadPoolError { source: ThreadPoolBuildError },
}