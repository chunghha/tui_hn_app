use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// A cache entry with expiration time
struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
}

/// Generic in-memory cache with TTL support
///
/// This cache supports an optional, config-driven metrics flag. By default
/// `Cache::new` creates a cache with metrics disabled to preserve the existing
/// constructor signature. To enable metrics, use `Cache::with_metrics`.
pub struct Cache<K, V> {
    entries: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    ttl: Duration,
    enable_metrics: bool,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Create a new cache with the given TTL (unused - prefer `with_metrics`)
    #[allow(dead_code)]
    pub fn new(ttl: Duration) -> Self {
        Self::with_metrics(ttl, false)
    }

    /// Create a new cache with the specified TTL and explicit metrics flag.
    pub fn with_metrics(ttl: Duration, enable_metrics: bool) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            ttl,
            enable_metrics,
        }
    }

    /// Get a value from the cache if it exists and hasn't expired.
    /// Emits a tracing debug log with elapsed time and hit/miss when metrics are enabled.
    pub fn get(&self, key: &K) -> Option<V> {
        let start = Instant::now();
        let entries = self.entries.read().ok()?;

        if let Some(entry) = entries.get(key)
            && Instant::now() < entry.expires_at
        {
            if self.enable_metrics {
                tracing::debug!(elapsed = ?start.elapsed(), hit = true, "cache.get");
            }
            return Some(entry.value.clone());
        }

        if self.enable_metrics {
            tracing::debug!(elapsed = ?start.elapsed(), hit = false, "cache.get");
        }
        None
    }

    /// Set a value in the cache. Emits a tracing debug log with elapsed time when enabled.
    pub fn set(&self, key: K, value: V) {
        let start = Instant::now();
        if let Ok(mut entries) = self.entries.write() {
            entries.insert(
                key,
                CacheEntry {
                    value,
                    expires_at: Instant::now() + self.ttl,
                },
            );
            if self.enable_metrics {
                tracing::debug!(elapsed = ?start.elapsed(), "cache.set");
            }
        } else if self.enable_metrics {
            tracing::debug!(elapsed = ?start.elapsed(), "cache.set failed (lock poisoned)");
        }
    }

    /// Invalidate (remove) a specific key. Emits a tracing debug log when enabled.
    #[allow(dead_code)]
    pub fn invalidate(&self, key: &K) {
        let start = Instant::now();
        if let Ok(mut entries) = self.entries.write() {
            entries.remove(key);
            if self.enable_metrics {
                tracing::debug!(elapsed = ?start.elapsed(), "cache.invalidate");
            }
        } else if self.enable_metrics {
            tracing::debug!(elapsed = ?start.elapsed(), "cache.invalidate failed (lock poisoned)");
        }
    }

    /// Clear all entries from the cache. Emits a tracing debug log when enabled.
    #[allow(dead_code)]
    pub fn clear(&self) {
        let start = Instant::now();
        if let Ok(mut entries) = self.entries.write() {
            entries.clear();
            if self.enable_metrics {
                tracing::debug!(elapsed = ?start.elapsed(), "cache.clear");
            }
        } else if self.enable_metrics {
            tracing::debug!(elapsed = ?start.elapsed(), "cache.clear failed (lock poisoned)");
        }
    }

    /// Remove expired entries from the cache. Emits a tracing debug log with counts when enabled.
    #[allow(dead_code)]
    pub fn cleanup_expired(&self) {
        let start = Instant::now();
        if let Ok(mut entries) = self.entries.write() {
            let before = entries.len();
            let now = Instant::now();
            entries.retain(|_, entry| now < entry.expires_at);
            let after = entries.len();
            if self.enable_metrics {
                tracing::debug!(elapsed = ?start.elapsed(), removed = before.saturating_sub(after), remaining = after, "cache.cleanup_expired");
            }
        } else if self.enable_metrics {
            tracing::debug!(elapsed = ?start.elapsed(), "cache.cleanup_expired failed (lock poisoned)");
        }
    }
}

impl<K, V> Clone for Cache<K, V> {
    fn clone(&self) -> Self {
        Self {
            entries: Arc::clone(&self.entries),
            ttl: self.ttl,
            enable_metrics: self.enable_metrics,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cache_set_and_get() {
        let cache = Cache::new(Duration::from_secs(60));
        cache.set(1, "hello".to_string());

        assert_eq!(cache.get(&1), Some("hello".to_string()));
        assert_eq!(cache.get(&2), None);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = Cache::new(Duration::from_millis(100));
        cache.set(1, "hello".to_string());

        // Should be available immediately
        assert_eq!(cache.get(&1), Some("hello".to_string()));

        // Wait for expiration
        thread::sleep(Duration::from_millis(150));

        // Should be expired
        assert_eq!(cache.get(&1), None);
    }

    #[test]
    fn test_cache_invalidate() {
        let cache = Cache::new(Duration::from_secs(60));
        cache.set(1, "hello".to_string());

        assert_eq!(cache.get(&1), Some("hello".to_string()));

        cache.invalidate(&1);
        assert_eq!(cache.get(&1), None);
    }

    #[test]
    fn test_cache_clear() {
        let cache = Cache::new(Duration::from_secs(60));
        cache.set(1, "hello".to_string());
        cache.set(2, "world".to_string());

        cache.clear();

        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), None);
    }

    #[test]
    fn test_cleanup_expired() {
        let cache = Cache::new(Duration::from_millis(100));
        cache.set(1, "expired".to_string());
        cache.set(2, "valid".to_string());

        // Wait for first entry to expire
        thread::sleep(Duration::from_millis(50));

        // Add a new entry with full TTL
        cache.set(3, "new".to_string());

        thread::sleep(Duration::from_millis(60));

        // Now entry 1 should be expired, 2 should be expired, 3 should be valid
        cache.cleanup_expired();

        // Verify by checking the internal state after cleanup
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&3), Some("new".to_string()));
    }
}
