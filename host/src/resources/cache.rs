//! # Texture Cache 模块
//!
//! 带 LRU 驱逐和显存预算的纹理缓存。

use crate::rendering_types::Texture;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

/// 默认显存预算：256 MB
pub const DEFAULT_TEXTURE_BUDGET_MB: usize = 256;

/// 缓存条目
struct CacheEntry {
    /// 纹理对象
    texture: Arc<dyn Texture>,
    /// 估算的显存占用（字节）
    size_bytes: usize,
    /// 引用计数（pin 状态）
    pin_count: u32,
}

impl CacheEntry {
    fn new(texture: Arc<dyn Texture>) -> Self {
        let size_bytes = texture.size_bytes();
        Self {
            texture,
            size_bytes,
            pin_count: 0,
        }
    }

    fn is_pinned(&self) -> bool {
        self.pin_count > 0
    }
}

/// 纹理缓存
///
/// 特性：
/// - LRU 驱逐策略
/// - 显存预算限制
/// - Pin/Unpin 支持（防止当前帧资源被驱逐）
pub struct TextureCache {
    /// 缓存条目（路径 -> 条目）
    entries: HashMap<String, CacheEntry>,
    /// LRU 顺序（最近使用的在后面）
    lru_order: VecDeque<String>,
    /// 显存预算（字节）
    budget_bytes: usize,
    /// 当前占用（字节）
    used_bytes: usize,
    /// 统计：命中次数
    hits: u64,
    /// 统计：未命中次数
    misses: u64,
    /// 统计：驱逐次数
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
            lru_order: VecDeque::new(),
            budget_bytes: budget_mb * 1024 * 1024,
            used_bytes: 0,
            hits: 0,
            misses: 0,
            evictions: 0,
        }
    }

    /// 使用默认预算创建缓存
    pub fn with_default_budget() -> Self {
        Self::new(DEFAULT_TEXTURE_BUDGET_MB)
    }

    /// 获取纹理（如果存在则更新 LRU）
    pub fn get(&mut self, key: &str) -> Option<Arc<dyn Texture>> {
        if let Some(entry) = self.entries.get(key) {
            self.hits += 1;
            let texture = Arc::clone(&entry.texture);
            self.touch(key);
            Some(texture)
        } else {
            self.misses += 1;
            None
        }
    }

    /// 只读获取纹理（不更新 LRU，用于渲染时快速查询）
    pub fn peek(&self, key: &str) -> Option<Arc<dyn Texture>> {
        self.entries.get(key).map(|e| Arc::clone(&e.texture))
    }

    /// 检查是否存在
    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// 插入纹理
    ///
    /// 如果超出预算，会先驱逐旧资源。
    pub fn insert(&mut self, key: String, texture: Arc<dyn Texture>) {
        let entry = CacheEntry::new(texture);
        let new_size = entry.size_bytes;

        // 如果已存在，先移除旧条目
        if let Some(old_entry) = self.entries.remove(&key) {
            self.used_bytes = self.used_bytes.saturating_sub(old_entry.size_bytes);
            self.remove_from_lru(&key);
        }

        // 驱逐直到有足够空间
        let mut eviction_attempts = 0;
        let max_attempts = self.entries.len();
        while self.used_bytes + new_size > self.budget_bytes {
            if !self.evict_one() {
                eviction_attempts += 1;
                if eviction_attempts >= max_attempts {
                    tracing::warn!(
                        used_mb = (self.used_bytes + new_size) as f64 / 1024.0 / 1024.0,
                        budget_mb = self.budget_bytes as f64 / 1024.0 / 1024.0,
                        "Texture cache over budget, cannot evict (all pinned?)"
                    );
                    break;
                }
            } else {
                eviction_attempts = 0;
            }
        }

        // 插入新条目
        self.used_bytes += new_size;
        self.entries.insert(key.clone(), entry);
        self.lru_order.push_back(key);
    }

    /// Pin 纹理（防止驱逐）
    pub fn pin(&mut self, key: &str) {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.pin_count += 1;
        }
    }

    /// Unpin 纹理
    pub fn unpin(&mut self, key: &str) {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.pin_count = entry.pin_count.saturating_sub(1);
        }
    }

    /// Unpin 所有纹理（帧结束时调用）
    pub fn unpin_all(&mut self) {
        for entry in self.entries.values_mut() {
            entry.pin_count = 0;
        }
    }

    /// 移除指定纹理
    pub fn remove(&mut self, key: &str) {
        if let Some(entry) = self.entries.remove(key) {
            self.used_bytes = self.used_bytes.saturating_sub(entry.size_bytes);
            self.remove_from_lru(key);
        }
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        self.entries.clear();
        self.lru_order.clear();
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
            hits: self.hits,
            misses: self.misses,
            evictions: self.evictions,
            hit_rate: if self.hits + self.misses > 0 {
                self.hits as f64 / (self.hits + self.misses) as f64
            } else {
                0.0
            },
        }
    }

    pub fn reset_stats(&mut self) {
        self.hits = 0;
        self.misses = 0;
        self.evictions = 0;
    }

    // === 内部方法 ===

    fn touch(&mut self, key: &str) {
        self.remove_from_lru(key);
        self.lru_order.push_back(key.to_string());
    }

    fn remove_from_lru(&mut self, key: &str) {
        self.lru_order.retain(|k| k != key);
    }

    fn evict_one(&mut self) -> bool {
        let key_to_evict = self
            .lru_order
            .iter()
            .find(|k| {
                self.entries
                    .get(*k)
                    .map(|e| !e.is_pinned())
                    .unwrap_or(false)
            })
            .cloned();

        if let Some(key) = key_to_evict
            && let Some(entry) = self.entries.remove(&key)
        {
            self.used_bytes = self.used_bytes.saturating_sub(entry.size_bytes);
            self.remove_from_lru(&key);
            self.evictions += 1;
            return true;
        }

        false
    }
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub used_bytes: usize,
    pub budget_bytes: usize,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub hit_rate: f64,
}

impl CacheStats {
    pub fn format(&self) -> String {
        format!(
            "Cache: {} entries, {:.1}MB / {:.1}MB ({:.1}%), hit rate: {:.1}%, evictions: {}",
            self.entries,
            self.used_bytes as f64 / 1024.0 / 1024.0,
            self.budget_bytes as f64 / 1024.0 / 1024.0,
            if self.budget_bytes > 0 {
                self.used_bytes as f64 / self.budget_bytes as f64 * 100.0
            } else {
                0.0
            },
            self.hit_rate * 100.0,
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
            hits: 80,
            misses: 20,
            evictions: 5,
            hit_rate: 0.8,
        };

        let formatted = stats.format();
        assert!(formatted.contains("10 entries"));
        assert!(formatted.contains("50.0MB"));
        assert!(formatted.contains("256.0MB"));
        assert!(formatted.contains("80.0%"));
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
