/// This module exists to avoid cloning header keys in returned HashMaps.
/// Since the underlying RString creation already involves cloning,
/// this caching layer aims to reduce redundant allocations.
///
/// Note: Performance testing on macOS showed minimal speed improvements,
/// so this optimization could be removed if any issues arise.
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, LazyLock, Mutex,
    },
};

use magnus::{r_string::FString, value::Opaque, IntoValue, RString, Ruby, Value};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Failed to acquire lock: {0}")]
    LockError(String),
}

static STRING_CACHE: LazyLock<Mutex<HashMap<&'static str, (Arc<StringCacheKey>, AtomicU32)>>> =
    LazyLock::new(|| Mutex::new(HashMap::with_capacity(100)));

pub struct StringCache;

pub struct StringCacheKey(Opaque<FString>, &'static str);

impl StringCacheKey {
    pub fn new(string: &str) -> Self {
        let rstr = RString::new(string);
        let fstr = rstr.to_interned_str();
        Self(Opaque::from(fstr), fstr.as_str().unwrap())
    }
}

impl AsRef<str> for StringCacheKey {
    fn as_ref(&self) -> &'static str {
        self.1
    }
}

impl IntoValue for StringCacheKey {
    fn into_value_with(self, handle: &Ruby) -> Value {
        handle.into_value(self.0)
    }
}

impl IntoValue for &StringCacheKey {
    fn into_value_with(self, handle: &Ruby) -> Value {
        handle.into_value(self.0)
    }
}

impl std::fmt::Debug for StringCacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.1.fmt(f)
    }
}

impl PartialEq for StringCacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl std::cmp::Eq for StringCacheKey {}

impl std::hash::Hash for StringCacheKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.1.hash(state);
    }
}

impl StringCache {
    #[allow(dead_code)]
    pub fn intern(string: String) -> Result<Arc<StringCacheKey>, CacheError> {
        let mut cache = STRING_CACHE
            .lock()
            .map_err(|e| CacheError::LockError(e.to_string()))?;

        if let Some((_, (interned_string, counter))) = cache.get_key_value(string.as_str()) {
            counter.fetch_add(1, Ordering::Relaxed);
            Ok(interned_string.clone())
        } else {
            let interned = Arc::new(StringCacheKey::new(string.as_str()));
            let leaked = Box::leak(string.into_boxed_str());
            cache.insert(leaked, (interned.clone(), AtomicU32::new(1)));
            Ok(interned)
        }
    }

    pub fn intern_many(strings: &[String]) -> Result<Vec<Arc<StringCacheKey>>, CacheError> {
        let mut cache = STRING_CACHE
            .lock()
            .map_err(|e| CacheError::LockError(e.to_string()))?;

        let mut result: Vec<Arc<StringCacheKey>> = Vec::with_capacity(strings.len());
        for string in strings {
            if let Some((_, (interned_string, counter))) = cache.get_key_value(string.as_str()) {
                counter.fetch_add(1, Ordering::Relaxed);
                result.push(interned_string.clone());
            } else {
                let interned = Arc::new(StringCacheKey::new(string));
                let leaked = Box::leak(string.clone().into_boxed_str());
                cache.insert(leaked, (interned.clone(), AtomicU32::new(1)));
                result.push(interned);
            }
        }
        Ok(result)
    }
}
