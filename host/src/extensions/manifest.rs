//! 扩展元信息定义。

/// Extension 元信息清单。
#[derive(Debug, Clone)]
pub struct ExtensionManifest {
    /// 扩展名（全局唯一）
    pub name: String,
    /// 扩展版本
    pub version: String,
    /// 兼容的引擎扩展 API 版本
    pub engine_api_version: String,
    /// 提供的 capability 列表
    pub capabilities: Vec<String>,
    /// 依赖的其他扩展（可选）
    pub dependencies: Vec<String>,
}

impl ExtensionManifest {
    /// 从 manifest 解析 engine_api 主版本号。
    pub fn engine_api_major(&self) -> Option<u64> {
        self.engine_api_version
            .split('.')
            .next()
            .and_then(|part| part.parse::<u64>().ok())
    }
}
