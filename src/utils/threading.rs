//! 线程工具

use std::sync::Arc;

use rayon::ThreadPool;

/// 后台任务线程池
#[allow(dead_code)]
pub struct ThreadPoolManager {
    pool: Arc<ThreadPool>,
}

impl ThreadPoolManager {
    /// 创建新的线程池管理器
    pub fn new(num_threads: usize) -> Self {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .expect("创建线程池失败");

        Self {
            pool: Arc::new(pool),
        }
    }

    /// 在线程池中执行任务
    pub fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.pool.spawn(f);
    }

    /// 获取线程池
    #[allow(dead_code)]
    pub fn pool(&self) -> &ThreadPool {
        &self.pool
    }
}

impl Default for ThreadPoolManager {
    fn default() -> Self {
        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        Self::new(num_threads)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_thread_pool_manager_new() {
        let manager = ThreadPoolManager::new(2);
        assert_eq!(manager.pool().current_num_threads(), 2);
    }

    #[test]
    fn test_thread_pool_manager_default() {
        let manager = ThreadPoolManager::default();
        assert!(manager.pool().current_num_threads() > 0);
    }

    #[test]
    fn test_thread_pool_manager_spawn() {
        let manager = ThreadPoolManager::new(2);
        let counter = Arc::new(AtomicUsize::new(0));

        let counter_clone = counter.clone();
        manager.spawn(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        std::thread::sleep(std::time::Duration::from_millis(50));

        assert!(counter.load(Ordering::SeqCst) >= 1);
    }

    #[test]
    fn test_thread_pool_manager_multiple_spawns() {
        let manager = ThreadPoolManager::new(4);
        let counter = Arc::new(AtomicUsize::new(0));

        for _ in 0..10 {
            let counter_clone = counter.clone();
            manager.spawn(move || {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            });
        }

        std::thread::sleep(std::time::Duration::from_millis(100));

        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_thread_pool_manager_pool_access() {
        let manager = ThreadPoolManager::new(2);
        let pool = manager.pool();
        assert_eq!(pool.current_num_threads(), 2);
    }
}
