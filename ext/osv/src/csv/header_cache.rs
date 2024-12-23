/// This module exists to avoid cloning header keys in returned HashMaps.
/// Since the underlying RString creation already involves cloning,
/// this caching layer aims to reduce redundant allocations.
///
/// Note: Performance testing on macOS showed minimal speed improvements,
/// so this optimization could be removed if any issues arise.


use std::{
    collections::HashMap,
    sync::{atomic::AtomicU32, LazyLock, Mutex},
};

static STRING_CACHE: LazyLock<Mutex<HashMap<&'static str, AtomicU32>>> =
    LazyLock::new(|| Mutex::new(HashMap::with_capacity(100)));

pub struct StringCache {}

impl StringCache {
    #[allow(dead_code)]
    pub fn intern(string: String) -> Result<&'static str, String> {
        let mut cache = STRING_CACHE.lock().map_err(|e| e.to_string())?;

        if let Some((&existing, count)) = cache.get_key_value(string.as_str()) {
            count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Ok(existing)
        } else {
            let leaked = Box::leak(string.into_boxed_str());
            cache.insert(leaked, AtomicU32::new(1));
            Ok(leaked)
        }
    }

    pub fn intern_many(strings: &[String]) -> Result<Vec<&'static str>, String> {
        let mut cache = STRING_CACHE.lock().map_err(|e| e.to_string())?;
        let mut result = Vec::with_capacity(strings.len());

        for string in strings {
            let static_str: &'static str =
                if let Some((&existing, count)) = cache.get_key_value(string.as_str()) {
                    count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    existing
                } else {
                    let leaked = Box::leak(string.clone().into_boxed_str());
                    cache.insert(leaked, AtomicU32::new(1));
                    leaked
                };
            result.push(static_str);
        }

        Ok(result)
    }

    pub fn clear(headers: &[&'static str]) -> Result<(), String> {
        let cache = STRING_CACHE.lock().map_err(|e| e.to_string())?;

        for header in headers {
            if let Some(count) = cache.get(header) {
                let remaining = count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                if remaining == 0 {
                    let ptr = *header as *const str as *mut str;
                    unsafe {
                        let _ = Box::from_raw(ptr);
                    }
                }
            }
        }

        Ok(())
    }
}
