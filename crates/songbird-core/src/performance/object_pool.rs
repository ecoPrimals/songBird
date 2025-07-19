//! Object pool for memory optimization

use std::sync::Arc;
use tokio::sync::Mutex;

/// High-performance object pool for memory optimization
pub struct ObjectPool<T> {
    /// Available objects in the pool
    objects: Arc<Mutex<Vec<T>>>,
    /// Factory function to create new objects
    factory: Arc<dyn Fn() -> T + Send + Sync>,
    /// Maximum pool size
    max_size: usize,
}

impl<T: Send + 'static> ObjectPool<T> {
    /// Create new object pool
    pub fn new<F>(factory: F, max_size: usize) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            objects: Arc::new(Mutex::new(Vec::with_capacity(max_size))),
            factory: Arc::new(factory),
            max_size,
        }
    }

    /// Get object from pool or create new one
    pub async fn get(&self) -> PooledObject<T> {
        let mut objects = self.objects.lock().await;

        let object = if let Some(obj) = objects.pop() {
            obj
        } else {
            (self.factory)()
        };

        PooledObject {
            object: Some(object),
            pool: self.objects.clone(),
            max_size: self.max_size,
        }
    }

    /// Get current pool size
    pub async fn size(&self) -> usize {
        self.objects.lock().await.len()
    }

    /// Check if pool is empty
    pub async fn is_empty(&self) -> bool {
        self.objects.lock().await.is_empty()
    }

    /// Clear the pool
    pub async fn clear(&self) {
        self.objects.lock().await.clear();
    }
}

/// RAII wrapper for pooled objects
pub struct PooledObject<T: Send + 'static> {
    object: Option<T>,
    pool: Arc<Mutex<Vec<T>>>,
    max_size: usize,
}

impl<T: Send + 'static> PooledObject<T> {
    /// Get mutable reference to the object
    pub fn get_mut(&mut self) -> &mut T {
        self.object.as_mut().expect("Object should be present")
    }

    /// Get reference to the object
    pub fn get(&self) -> &T {
        self.object.as_ref().expect("Object should be present")
    }
}

impl<T: Send + 'static> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(object) = self.object.take() {
            // Return object to pool if pool isn't full
            let pool = self.pool.clone();
            let max_size = self.max_size;

            tokio::spawn(async move {
                let mut objects = pool.lock().await;
                if objects.len() < max_size {
                    objects.push(object);
                }
                // Otherwise, object is dropped naturally
            });
        }
    }
}

impl<T: Send + 'static> std::ops::Deref for PooledObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T: Send + 'static> std::ops::DerefMut for PooledObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

/// Specialized string buffer pool
pub type StringBufferPool = ObjectPool<String>;

impl StringBufferPool {
    /// Create new string buffer pool
    pub fn new_string_pool(max_size: usize) -> Self {
        Self::new(|| String::with_capacity(1024), max_size)
    }
}

/// Specialized vector buffer pool
pub type VecBufferPool<T> = ObjectPool<Vec<T>>;

impl<T: Send + 'static> VecBufferPool<T> {
    /// Create new vector buffer pool
    pub fn new_vec_pool(initial_capacity: usize, max_size: usize) -> Self {
        Self::new(move || Vec::with_capacity(initial_capacity), max_size)
    }
}

/// Specialized byte buffer pool
pub type ByteBufferPool = ObjectPool<Vec<u8>>;

impl ByteBufferPool {
    /// Create new byte buffer pool
    pub fn new_byte_pool(buffer_size: usize, max_size: usize) -> Self {
        Self::new(move || Vec::with_capacity(buffer_size), max_size)
    }

    /// Get buffer with guaranteed minimum capacity
    pub async fn get_with_capacity(&self, min_capacity: usize) -> PooledObject<Vec<u8>> {
        let mut obj = self.get().await;
        obj.get_mut().reserve(min_capacity);
        obj.get_mut().clear(); // Clear previous data
        obj
    }
}
