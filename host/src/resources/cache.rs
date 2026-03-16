//! # Texture Cache 模块
//!
//! 带显存预算的纹理缓存，FIFO 驱逐（先插入的先逐出）。

use crate::rendering_types::Texture;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

/// 默认显存预算：256 MB
pub const DEFAULT_TEXTURE_BUDGET_MB: usize = 256;

/// 缓存条目
struct CacheEntry {
    texture: Arc<dyn Texture>,
    size_bytes: usize,
}

impl CacheEntry {
    fn new(texture: Arc<dyn Texture>) -> Self {
        let size_bytes = texture.size_bytes();
        Self {
            texture,
            size_bytes,
        }
    }
}

/// 纹理缓存
///
/// - FIFO 驱逐：超预算时逐出最早插入的条目
/// - 显存预算限制
pub struct TextureCache {
    entries: HashMap<String, CacheEntry>,
    /// 插入顺序（队头最早），用于 FIFO 驱逐
    fifo_order: VecDeque<String>,
    budget_bytes: usize,
    used_bytes: usize,
    evictions: u64,
}

impl TextureCache {
    /// 创建新的纹理缓存
    ///
    /// # 参数
    /// - `budget_mb`: 显存预算（MB）
    pub fn new(budget_mb: usize) -> Self {
        Self {
            entries: HashMap::new(),
            fifo_order: VecDeque::new(),
            budget_bytes: budget_mb * 1024 * 1024,
            used_bytes: 0,
            evictions: 0,
        }
    }

    /// 使用默认预算创建缓存
    pub fn with_default_budget() -> Self {
        Self::new(DEFAULT_TEXTURE_BUDGET_MB)
    }

    /// 只读获取纹理
    pub fn get(&self, key: &str) -> Option<Arc<dyn Texture>> {
        self.entries.get(key).map(|e| Arc::clone(&e.texture))
    }

    /// 检查是否存在
    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// 插入纹理。若超出预算，按 FIFO 逐出旧条目直至有足够空间。
    pub fn insert(&mut self, key: String, texture: Arc<dyn Texture>) {
        let entry = CacheEntry::new(texture);
        let new_size = entry.size_bytes;

        if let Some(old_entry) = self.entries.remove(&key) {
            self.used_bytes = self.used_bytes.saturating_sub(old_entry.size_bytes);
            self.fifo_remove(&key);
        }

        while self.used_bytes + new_size > self.budget_bytes {
            if !self.evict_one() {
                tracing::warn!(
                    used_mb = (self.used_bytes + new_size) as f64 / 1024.0 / 1024.0,
                    budget_mb = self.budget_bytes as f64 / 1024.0 / 1024.0,
                    "Texture cache over budget, eviction exhausted"
                );
                break;
            }
        }

        self.used_bytes += new_size;
        self.entries.insert(key.clone(), entry);
        self.fifo_order.push_back(key);
    }

    /// 移除指定纹理
    pub fn remove(&mut self, key: &str) {
        if let Some(entry) = self.entries.remove(key) {
            self.used_bytes = self.used_bytes.saturating_sub(entry.size_bytes);
            self.fifo_remove(key);
        }
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        self.entries.clear();
        self.fifo_order.clear();
        self.used_bytes = 0;
    }

    pub fn used_bytes(&self) -> usize {
        self.used_bytes
    }

    pub fn budget_bytes(&self) -> usize {
        self.budget_bytes
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 获取统计信息
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.entries.len(),
            used_bytes: self.used_bytes,
            budget_bytes: self.budget_bytes,
            evictions: self.evictions,
        }
    }

    fn fifo_remove(&mut self, key: &str) {
        self.fifo_order.retain(|k| k != key);
    }

    fn evict_one(&mut self) -> bool {
        let key = match self.fifo_order.pop_front() {
            Some(k) => k,
            None => return false,
        };
        if let Some(entry) = self.entries.remove(&key) {
            self.used_bytes = self.used_bytes.saturating_sub(entry.size_bytes);
            self.evictions += 1;
        }
        true
    }
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub used_bytes: usize,
    pub budget_bytes: usize,
    pub evictions: u64,
}

impl CacheStats {
    pub fn format(&self) -> String {
        format!(
            "Cache: {} entries, {:.1}MB / {:.1}MB ({:.1}%), evictions: {}",
            self.entries,
            self.used_bytes as f64 / 1024.0 / 1024.0,
            self.budget_bytes as f64 / 1024.0 / 1024.0,
            if self.budget_bytes > 0 {
                self.used_bytes as f64 / self.budget_bytes as f64 * 100.0
            } else {
                0.0
            },
            self.evictions,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_stats_format() {
        let stats = CacheStats {
            entries: 10,
            used_bytes: 50 * 1024 * 1024,
            budget_bytes: 256 * 1024 * 1024,
            evictions: 5,
        };

        let formatted = stats.format();
        assert!(formatted.contains("10 entries"));
        assert!(formatted.contains("50.0MB"));
        assert!(formatted.contains("256.0MB"));
        assert!(formatted.contains("evictions: 5"));
    }

    #[test]
    fn test_cache_budget() {
        let cache = TextureCache::new(128);
        assert_eq!(cache.budget_bytes(), 128 * 1024 * 1024);
        assert_eq!(cache.used_bytes(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_default_budget() {
        let cache = TextureCache::with_default_budget();
        assert_eq!(
            cache.budget_bytes(),
            DEFAULT_TEXTURE_BUDGET_MB * 1024 * 1024
        );
    }

    fn null_texture(w: u32, h: u32) -> Arc<dyn Texture> {
        use crate::rendering_types::NullTexture;
        Arc::new(NullTexture::new(w, h))
    }

    #[test]
    fn test_insert_and_get() {
        let mut cache = TextureCache::new(256);
        let tex = null_texture(64, 64);
        cache.insert("bg.png".to_string(), Arc::clone(&tex));

        assert!(cache.contains("bg.png"));
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.used_bytes(), 64 * 64 * 4);

        let got = cache.get("bg.png").unwrap();
        assert_eq!(got.width_u32(), 64);
    }

    #[test]
    fn test_remove() {
        let mut cache = TextureCache::new(256);
        cache.insert("a.png".to_string(), null_texture(32, 32));
        assert!(cache.contains("a.png"));

        cache.remove("a.png");
        assert!(!cache.contains("a.png"));
        assert_eq!(cache.used_bytes(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_remove_nonexistent_is_noop() {
        let mut cache = TextureCache::new(256);
        cache.remove("ghost.png"); // should not panic
        assert!(cache.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut cache = TextureCache::new(256);
        cache.insert("a.png".to_string(), null_texture(32, 32));
        cache.insert("b.png".to_string(), null_texture(64, 64));
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.used_bytes(), 0);
    }

    #[test]
    fn test_fifo_eviction() {
        // budget = 2 textures of 4 bytes each (2x2x4 = 16 bytes), budget set to 16 bytes
        // But TextureCache::new takes MB, so use a very small budget via a direct-bytes workaround.
        // Use 1 MB budget but insert textures that together exceed it.
        // Create budget that holds exactly 2 textures of size 1x1 (4 bytes each) = 8 bytes
        // We need budget in MB; smallest is 1 MB. Instead, track that eviction happens with large textures.

        // Use 1 MB budget; insert textures of ~600 KB each; third insert should evict first.
        let budget_mb = 1usize;
        let mut cache = TextureCache::new(budget_mb);
        // Each NullTexture(512,300) → 512*300*4 = 614400 bytes ≈ 0.586 MB
        let w = 512u32;
        let h = 300u32;

        cache.insert("first".to_string(), null_texture(w, h));
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.stats().evictions, 0);

        cache.insert("second".to_string(), null_texture(w, h));
        // Total: 2 * 614400 = 1228800 > 1MB; first should be evicted
        assert_eq!(cache.len(), 1, "first entry should have been evicted");
        assert!(!cache.contains("first"));
        assert!(cache.contains("second"));
        assert_eq!(cache.stats().evictions, 1);
    }

    #[test]
    fn test_overwrite_same_key_updates_size() {
        let mut cache = TextureCache::new(256);
        cache.insert("img.png".to_string(), null_texture(64, 64));
        let old_bytes = cache.used_bytes();

        cache.insert("img.png".to_string(), null_texture(128, 128));
        let new_bytes = cache.used_bytes();

        assert_ne!(old_bytes, new_bytes);
        assert_eq!(new_bytes, 128 * 128 * 4);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_stats_eviction_count() {
        let budget_mb = 1usize;
        let mut cache = TextureCache::new(budget_mb);

        // Insert 3 textures, each ~0.586MB → evictions happen
        for i in 0..3 {
            cache.insert(format!("tex_{i}"), null_texture(512, 300));
        }

        assert!(cache.stats().evictions >= 2);
    }
}
