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
        LazyLock, Mutex,
    },
};

use magnus::{IntoValue, RString, Ruby, Value};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Failed to acquire lock: {0}")]
    LockError(String),
}

static STRING_CACHE: LazyLock<Mutex<HashMap<&'static str, (StringCacheKey, AtomicU32)>>> =
    LazyLock::new(|| Mutex::new(HashMap::with_capacity(100)));

pub struct StringCache;

#[derive(Copy, Clone)]
pub struct StringCacheKey(&'static str);

impl StringCacheKey {
    pub fn new(string: &str) -> Self {
        let rstr = RString::new(string);
        let fstr = rstr.to_interned_str();
        Self(fstr.as_str().unwrap())
    }
}

impl AsRef<str> for StringCacheKey {
    fn as_ref(&self) -> &'static str {
        self.0
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
        self.0.fmt(f)
    }
}

impl PartialEq for StringCacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl std::cmp::Eq for StringCacheKey {}

impl std::hash::Hash for StringCacheKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl StringCache {
    pub fn intern_many<AsStr: AsRef<str>>(
        strings: &[AsStr],
    ) -> Result<Vec<StringCacheKey>, CacheError> {
        let mut cache = STRING_CACHE
            .lock()
            .map_err(|e| CacheError::LockError(e.to_string()))?;

        let mut result: Vec<StringCacheKey> = Vec::with_capacity(strings.len());
        for string in strings {
            if let Some((_, (interned_string, counter))) = cache.get_key_value(string.as_ref()) {
                counter.fetch_add(1, Ordering::Relaxed);
                result.push(*interned_string);
            } else {
                let interned = StringCacheKey::new(string.as_ref());
                cache.insert(interned.0, (interned, AtomicU32::new(1)));
                result.push(interned);
            }
        }
        Ok(result)
    }
}
