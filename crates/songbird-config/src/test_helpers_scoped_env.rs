//! Scoped environment variable helper for automatic cleanup

use std::sync::MutexGuard;

/// RAII guard for scoped environment variable setting/removal
///
/// Automatically cleans up environment variables when dropped.
pub struct ScopedEnv {
    key: String,
    old_value: Option<String>,
    _parent_guard: Option<MutexGuard<'static, ()>>,
}

impl ScopedEnv {
    /// Set an environment variable, restoring the old value on drop
    pub fn new<K: Into<String>, V: AsRef<str>>(key: K, value: V) -> Self {
        let key = key.into();
        let old_value = std::env::var(&key).ok();
        std::env::set_var(&key, value.as_ref());
        
        Self {
            key,
            old_value,
            _parent_guard: None,
        }
    }
    
    /// Remove an environment variable, restoring it on drop
    pub fn remove<K: Into<String>>(key: K) -> Self {
        let key = key.into();
        let old_value = std::env::var(&key).ok();
        std::env::remove_var(&key);
        
        Self {
            key,
            old_value,
            _parent_guard: None,
        }
    }
}

impl Drop for ScopedEnv {
    fn drop(&mut self) {
        match &self.old_value {
            Some(value) => std::env::set_var(&self.key, value),
            None => std::env::remove_var(&self.key),
        }
    }
}

