//! # Texture Cache 模块
//!
//! 带 LRU 驱逐和显存预算的纹理缓存。

use macroquad::prelude::*;
use std::collections::{HashMap, VecDeque};

/// 默认显存预算：256 MB
pub const DEFAULT_TEXTURE_BUDGET_MB: usize = 256;

/// 缓存条目
#[derive(Debug)]
struct CacheEntry {
    /// 纹理对象
    texture: Texture2D,
    /// 估算的显存占用（字节）
    size_bytes: usize,
    /// 引用计数（pin 状态）
    pin_count: u32,
}

impl CacheEntry {
    fn new(texture: Texture2D) -> Self {
        // 估算纹理显存：width * height * 4 (RGBA8)
        let size_bytes = (texture.width() as usize) * (texture.height() as usize) * 4;
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
    pub fn get(&mut self, key: &str) -> Option<Texture2D> {
        if self.entries.contains_key(key) {
            self.hits += 1;
            self.touch(key);
            Some(self.entries.get(key).unwrap().texture.clone())
        } else {
            self.misses += 1;
            None
        }
    }

    /// 只读获取纹理（不更新 LRU，用于渲染时快速查询）
    pub fn peek(&self, key: &str) -> Option<Texture2D> {
        self.entries.get(key).map(|e| e.texture.clone())
    }

    /// 检查是否存在
    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// 插入纹理
    ///
    /// 如果超出预算，会先驱逐旧资源。
    pub fn insert(&mut self, key: String, texture: Texture2D) {
        let entry = CacheEntry::new(texture);
        let new_size = entry.size_bytes;

        // 如果已存在，先移除旧条目
        if let Some(old_entry) = self.entries.remove(&key) {
            self.used_bytes = self.used_bytes.saturating_sub(old_entry.size_bytes);
            self.remove_from_lru(&key);
        }

        // 驱逐直到有足够空间
        let mut eviction_attempts = 0;
        let max_attempts = self.entries.len(); // 最多尝试驱逐所有条目
        while self.used_bytes + new_size > self.budget_bytes {
            if !self.evict_one() {
                // 无法驱逐更多（可能全部被 pin 或队列为空）
                eviction_attempts += 1;
                if eviction_attempts >= max_attempts {
                    eprintln!(
                        "⚠️ 警告：纹理缓存超出预算（{:.1}MB / {:.1}MB），但无法驱逐资源（可能全部被 pin）。强制插入新资源可能导致显存溢出。",
                        (self.used_bytes + new_size) as f64 / 1024.0 / 1024.0,
                        self.budget_bytes as f64 / 1024.0 / 1024.0
                    );
                    break;
                }
            } else {
                // 成功驱逐，重置尝试计数
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

    /// 获取当前占用（字节）
    pub fn used_bytes(&self) -> usize {
        self.used_bytes
    }

    /// 获取预算（字节）
    pub fn budget_bytes(&self) -> usize {
        self.budget_bytes
    }

    /// 获取缓存条目数量
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 缓存是否为空
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

    /// 重置统计
    pub fn reset_stats(&mut self) {
        self.hits = 0;
        self.misses = 0;
        self.evictions = 0;
    }

    // === 内部方法 ===

    /// 更新 LRU 顺序（将 key 移到最后）
    fn touch(&mut self, key: &str) {
        self.remove_from_lru(key);
        self.lru_order.push_back(key.to_string());
    }

    /// 从 LRU 列表中移除
    fn remove_from_lru(&mut self, key: &str) {
        self.lru_order.retain(|k| k != key);
    }

    /// 驱逐一个资源（LRU，跳过 pinned）
    ///
    /// 返回是否成功驱逐
    fn evict_one(&mut self) -> bool {
        // 找到第一个未 pin 的资源
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
    /// 缓存条目数量
    pub entries: usize,
    /// 当前占用（字节）
    pub used_bytes: usize,
    /// 预算（字节）
    pub budget_bytes: usize,
    /// 命中次数
    pub hits: u64,
    /// 未命中次数
    pub misses: u64,
    /// 驱逐次数
    pub evictions: u64,
    /// 命中率
    pub hit_rate: f64,
}

impl CacheStats {
    /// 格式化为可读字符串
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

    // 创建一个模拟纹理（用于测试）
    // 注意：实际测试中需要 macroquad 上下文，这里只测试逻辑

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
        assert!(formatted.contains("80.0%")); // hit rate
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
