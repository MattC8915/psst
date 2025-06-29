use std::{
    collections::hash_map::DefaultHasher,
    fs::{self, File},
    hash::{Hash, Hasher},
    num::NonZeroUsize,
    path::PathBuf,
    sync::Arc,
};

use druid::image;
use druid::ImageBuf;
use lru::LruCache;
use parking_lot::Mutex;
use psst_core::cache::mkdir_if_not_exists;

pub struct WebApiCache {
    base: Option<PathBuf>,
    images: Mutex<LruCache<Arc<str>, ImageBuf>>,
    // playlists: Mutex<LruCache<Arc<str>, ImageBuf>>,
}

impl WebApiCache {
    pub fn new(base: Option<PathBuf>) -> Self {
        const IMAGE_CACHE_SIZE: usize = 256;
        Self {
            base,
            images: Mutex::new(LruCache::new(NonZeroUsize::new(IMAGE_CACHE_SIZE).unwrap())),
            // playlists: Mutex::new(LruCache::new(NonZeroUsize::new(IMAGE_CACHE_SIZE).unwrap())),
        }
    }

    pub fn get_image(&self, uri: &Arc<str>) -> Option<ImageBuf> {
        self.images.lock().get(uri).cloned()
    }

    pub fn set_image(&self, uri: Arc<str>, image: ImageBuf) {
        self.images.lock().put(uri, image);
    }

    pub fn get_image_from_disk(&self, uri: &Arc<str>) -> Option<ImageBuf> {
        let hash = Self::hash_uri(uri);
        self.key("images", &format!("{:016x}", hash))
            .and_then(|path| std::fs::read(path).ok())
            .and_then(|bytes| image::load_from_memory(&bytes).ok())
            .map(ImageBuf::from_dynamic_image)
    }

    pub fn save_image_to_disk(&self, uri: &Arc<str>, data: &[u8]) {
        let hash = Self::hash_uri(uri);
        if let Some(path) = self.key("images", &format!("{:016x}", hash)) {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
                // log::info!("Creating dir path: {}", parent.display());
            }
            log::info!("Saving image to disk: {}", path.display());
            let _ = std::fs::write(path, data);
        }
    }

    fn hash_uri(uri: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        uri.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get(&self, bucket: &str, key: &str) -> Option<File> {
        self.key(bucket, key).and_then(|path| File::open(path).ok())
    }

    pub fn set(&self, bucket: &str, key: &str, value: &[u8]) {
        if let Some(path) = self.bucket(bucket) {
            if let Err(err) = mkdir_if_not_exists(&path) {
                log::error!("failed to create WebAPI cache bucket: {:?}", err);
            }
        }
        if let Some(path) = self.key(bucket, key) {
            if let Err(err) = fs::write(path, value) {
                log::error!("failed to save to WebAPI cache: {:?}", err);
            }
        }
    }

    /// Get cache statistics including size and entry counts
    pub fn get_stats(&self) -> CacheStats {
        let mut total_size = 0u64;
        let mut total_entries = 0u64;

        if let Some(base) = &self.base {
            if let Ok(entries) = fs::read_dir(base) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Ok(metadata) = fs::metadata(&path) {
                            total_size += metadata.len();
                            total_entries += 1;
                        }
                    } else if path.is_dir() {
                        if let Ok(dir_entries) = fs::read_dir(path) {
                            for dir_entry in dir_entries.flatten() {
                                if let Ok(metadata) = fs::metadata(dir_entry.path()) {
                                    total_size += metadata.len();
                                    total_entries += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        let image_cache_size = self.images.lock().len() as u64;

        CacheStats {
            total_size,
            total_entries,
            image_cache_entries: image_cache_size,
        }
    }

    /// Clear all cache entries
    pub fn clear_all(&self) -> Result<(), std::io::Error> {
        if let Some(base) = &self.base {
            if base.exists() {
                fs::remove_dir_all(base)?;
                mkdir_if_not_exists(base)?;
            }
        }
        self.images.lock().clear();
        Ok(())
    }

    /// Clear a specific bucket
    pub fn clear_bucket(&self, bucket: &str) -> Result<(), std::io::Error> {
        if let Some(bucket_path) = self.bucket(bucket) {
            if bucket_path.exists() {
                fs::remove_dir_all(bucket_path)?;
                mkdir_if_not_exists(&self.bucket(bucket).unwrap())?;
            }
        }
        Ok(())
    }

    fn bucket(&self, bucket: &str) -> Option<PathBuf> {
        self.base.as_ref().map(|path| path.join(bucket))
    }

    fn key(&self, bucket: &str, key: &str) -> Option<PathBuf> {
        self.bucket(bucket).map(|path| path.join(key))
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_size: u64,
    pub total_entries: u64,
    pub image_cache_entries: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_cache_basic_operations() {
        let temp_dir = tempdir().unwrap();
        let cache = WebApiCache::new(Some(temp_dir.path().to_path_buf()));

        // Test setting and getting data
        let test_data = b"test data";
        cache.set("test-bucket", "test-key", test_data);

        // Verify data was stored
        let stats = cache.get_stats();
        assert!(stats.total_entries > 0);
        assert!(stats.total_size > 0);

        // Test getting data
        if let Some(file) = cache.get("test-bucket", "test-key") {
            let mut content = Vec::new();
            use std::io::Read;
            file.take(100).read_to_end(&mut content).unwrap();
            assert_eq!(content, test_data);
        } else {
            panic!("Failed to retrieve cached data");
        }

        // Test clearing bucket
        cache.clear_bucket("test-bucket").unwrap();
        let stats_after_clear = cache.get_stats();
        assert_eq!(stats_after_clear.total_entries, 0);
    }

    #[test]
    fn test_cache_clear_all() {
        let temp_dir = tempdir().unwrap();
        let cache = WebApiCache::new(Some(temp_dir.path().to_path_buf()));

        // Add some test data
        cache.set("bucket1", "key1", b"data1");
        cache.set("bucket2", "key2", b"data2");

        let stats = cache.get_stats();
        assert!(stats.total_entries > 0);

        // Clear all
        cache.clear_all().unwrap();

        let stats_after_clear = cache.get_stats();
        assert_eq!(stats_after_clear.total_entries, 0);
        assert_eq!(stats_after_clear.total_size, 0);
    }
}
