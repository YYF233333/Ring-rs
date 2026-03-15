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
}
