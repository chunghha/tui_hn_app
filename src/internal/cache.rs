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
pub struct Cache<K, V> {
    entries: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    ttl: Duration,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Create a new cache with the specified TTL
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    /// Get a value from the cache if it exists and hasn't expired
    pub fn get(&self, key: &K) -> Option<V> {
        let entries = self.entries.read().ok()?;

        if let Some(entry) = entries.get(key)
            && Instant::now() < entry.expires_at
        {
            return Some(entry.value.clone());
        }
        None
    }

    /// Set a value in the cache
    pub fn set(&self, key: K, value: V) {
        if let Ok(mut entries) = self.entries.write() {
            entries.insert(
                key,
                CacheEntry {
                    value,
                    expires_at: Instant::now() + self.ttl,
                },
            );
        }
    }

    /// Invalidate (remove) a specific key
    #[allow(dead_code)]
    pub fn invalidate(&self, key: &K) {
        if let Ok(mut entries) = self.entries.write() {
            entries.remove(key);
        }
    }

    /// Clear all entries from the cache
    #[allow(dead_code)]
    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.clear();
        }
    }

    /// Remove expired entries from the cache
    #[allow(dead_code)]
    pub fn cleanup_expired(&self) {
        if let Ok(mut entries) = self.entries.write() {
            let now = Instant::now();
            entries.retain(|_, entry| now < entry.expires_at);
        }
    }
}

impl<K, V> Clone for Cache<K, V> {
    fn clone(&self) -> Self {
        Self {
            entries: Arc::clone(&self.entries),
            ttl: self.ttl,
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
