/// This module exists to avoid cloning header keys in returned HashMaps.
/// Since the underlying RString creation already involves cloning,
/// this caching layer aims to reduce redundant allocations.
///
/// Note: Performance testing on macOS showed minimal speed improvements,
/// so this optimization could be removed if any issues arise.
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use magnus::{
    r_string::FString,
    value::{InnerValue, Opaque},
    IntoValue, RString, Ruby, Value,
};

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum CacheError {
    #[error("Failed to acquire lock: {0}")]
    LockError(String),
    #[error("Failed to convert Ruby String to interned string: {0}")]
    RStringConversion(String),
}

static STRING_CACHE: LazyLock<Mutex<HashMap<String, StringCacheKey>>> =
    LazyLock::new(|| Mutex::new(HashMap::with_capacity(100)));

pub struct StringCache;

#[derive(Copy, Clone)]
pub struct StringCacheKey(Opaque<FString>);

impl StringCacheKey {
    pub fn new(string: &str) -> Result<Self, CacheError> {
        let rstr = RString::new(string);
        let fstr = rstr.to_interned_str();
        // FStrings should not be collected by the GC anyway, but just in case.
        magnus::gc::register_mark_object(fstr);
        Ok(Self(Opaque::from(fstr)))
    }

    pub fn as_fstr(&self, handle: &Ruby) -> FString {
        self.0.get_inner_with(handle)
    }

    pub fn as_str(&self, handle: &Ruby) -> Result<&'static str, CacheError> {
        self.0
            .get_inner_with(handle)
            .as_str()
            .map_err(|e| CacheError::RStringConversion(e.to_string()))
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

impl StringCache {
    pub fn intern_many<AsStr: AsRef<str>>(
        strings: &[AsStr],
    ) -> Result<Vec<StringCacheKey>, CacheError> {
        let mut cache = STRING_CACHE
            .lock()
            .map_err(|e| CacheError::LockError(e.to_string()))?;

        let mut result: Vec<StringCacheKey> = Vec::with_capacity(strings.len());
        for string in strings {
            if let Some((_, interned_string)) = cache.get_key_value(string.as_ref()) {
                result.push(*interned_string);
            } else {
                let interned = StringCacheKey::new(string.as_ref())?;
                cache.insert(string.as_ref().to_string(), interned);
                result.push(interned);
            }
        }
        Ok(result)
    }
}
