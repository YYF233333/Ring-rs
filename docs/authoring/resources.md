# 资源管理系统使用指南

本指南介绍如何使用 Ring Engine 的资源动态加载系统，包括按需加载、缓存管理、资源打包等功能。

## 目录

- [概述](#概述)
- [配置资源来源](#配置资源来源)
- [缓存系统](#缓存系统)
- [资源打包](#资源打包)
- [常见问题](#常见问题)

---

## 概述

资源管理系统提供了以下核心功能：

1. **动态加载**：启动时不再预加载所有资源，运行时按需加载
2. **LRU 缓存**：自动管理显存，超出缓存大小时驱逐最久未使用的资源
3. **多来源支持**：支持从文件系统或 ZIP 包加载资源
4. **资源打包**：提供工具将资源打包为 ZIP 文件，便于发布

### 核心优势

- ✅ **显存控制**：默认 256MB 缓存大小，避免显存溢出
- ✅ **启动速度**：不再需要等待所有资源加载完成
- ✅ **发布友好**：支持将资源打包为单个 ZIP 文件
- ✅ **开发便捷**：开发时仍可使用文件系统，无需打包

---

## 配置资源来源

### 文件系统模式（开发模式，默认）

在 `config.json` 中配置：

```json
{
  "assets_root": "assets",
  "asset_source": "fs"
}
```

**特点**：
- 直接从 `assets_root` 目录读取资源
- 适合开发和调试
- 修改资源后无需重新打包

### ZIP 模式（发布模式）

在 `config.json` 中配置：

```json
{
  "assets_root": "assets",
  "asset_source": "zip",
  "zip_path": "game.zip"
}
```

**特点**：
- 从 ZIP 文件中读取资源
- 适合发布，资源集中在一个文件
- 启动时会验证 ZIP 文件是否存在

其它 `config.json` 字段（如 `start_script_path` / `manifest_path` / `window` / `resources`）请直接参考：[config 配置说明](config.md)。

---

## 缓存系统

### 工作原理

资源管理系统使用 **LRU（Least Recently Used）** 缓存，首次使用资源时从磁盘/ZIP 加载到显存中，默认 256MB 缓存大小，超出缓存大小时，自动驱逐最久未使用的资源

### 缓存行为

- **当前帧资源**：正在显示的背景/立绘会被保护，不会被驱逐
- **自动管理**：无需手动管理，系统自动处理

### 资源路径规则

所有资源路径使用**相对路径**，相对于 `assets_root`：

```
✅ 正确：
backgrounds/bg1.jpg
characters/角色A.png
scripts/chapter1.md

❌ 错误：
/assets/backgrounds/bg1.jpg  (不要包含 assets_root)
C:\path\to\bg1.jpg            (不要使用绝对路径)
```

路径会自动规范化（处理 `../` 和 `./`），确保缓存键一致。

---

## 资源打包

### 使用打包工具

项目提供了 `asset-packer` 工具，用于将资源目录打包为 ZIP 文件。

#### 使用工具

**方式一：使用 cargo 运行（推荐，无需安装）**

在项目根目录直接运行：

```bash
# 打包资源（使用默认值：assets -> game.zip）
cargo run -p asset-packer

# 指定输入输出
cargo run -p asset-packer -- --input assets --output game.zip

# 列出 ZIP 内容
cargo run -p asset-packer -- list game.zip

# 验证 ZIP 完整性
cargo run -p asset-packer -- verify game.zip --input assets
```

> **提示**：`-p` 是 `--package` 的简写，`default-run` 已配置，无需指定 `--bin packer`。

**方式二：安装后使用**

```bash
# 安装工具
cargo install --path tools/asset-packer

# 使用（安装后可以直接使用 packer 命令）
packer
packer --input assets --output game.zip
packer list game.zip
packer verify game.zip --input assets
```

#### 打包资源

```bash
# 基本用法：打包 assets 目录到 game.zip（使用默认值）
cargo run -p asset-packer

# 指定输入输出
cargo run -p asset-packer -- --input assets --output game.zip
```

#### 列出 ZIP 内容

```bash
cargo run -p asset-packer -- list game.zip
```

输出示例：
```
game.zip 内容：
  backgrounds/bg1.jpg (1.2 MB)
  backgrounds/bg2.jpg (0.8 MB)
  characters/角色A.png (500 KB)
  scripts/chapter1.md (10 KB)
  ...
总计：50 个文件，25.3 MB
```

#### 验证 ZIP 完整性

```bash
# 验证 ZIP 文件与源目录是否一致
cargo run -p asset-packer -- verify game.zip --input assets
```

验证会检查：
- ZIP 中的文件是否存在于源目录
- 文件大小是否一致
- 文件内容是否匹配（可选哈希校验）

### 发布流程

1. **开发阶段**：使用文件系统模式（`asset_source: "fs"`）
2. **打包资源**：
   - **推荐的一键发行**：使用 `cargo pack release --output-dir dist --zip`（会自动生成 ZIP 模式的 `dist/config.json`）
   - **手工打包**：运行 `cargo run -p asset-packer` 生成 `game.zip`
3. **更新配置（仅手工打包需要）**：修改 `config.json`，设置 `asset_source: "zip"` 和 `zip_path: "game.zip"`
4. **测试发布**：删除或重命名 `assets` 目录，运行游戏验证 ZIP 模式
5. **分发**：分发 `exe` + `game.zip` + `config.json`（或直接分发 `dist/` 目录产物）

### 发布目录结构

```
发布目录/
├── host.exe              # 游戏可执行文件
├── game.zip              # 资源包（包含所有 assets）
└── config.json            # 配置文件
```

**注意**：发布时不需要 `assets/` 目录，所有资源都在 `game.zip` 中。

---

## 常见问题

### Q: 如何调整纹理缓存大小？

**A**: 在 `config.json` 中配置 `resources.texture_cache_size_mb` 字段：

```json
{
  "resources": {
    "texture_cache_size_mb": 512
  }
}
```

默认值为 256MB。可以根据你的游戏资源规模和目标设备显存调整。

### Q: ZIP 模式下资源加载失败？

**A**: 检查以下几点：

1. **ZIP 文件是否存在**：确认 `zip_path` 配置正确
2. **ZIP 文件完整性**：运行 `cargo run -p asset-packer -- verify game.zip --input assets` 验证
3. **路径一致性**：确保 ZIP 内的路径结构与文件系统模式一致

### Q: 资源路径中的 `../` 会被处理吗？

**A**: 会的。系统会自动规范化路径，`../` 和 `./` 会被正确处理。例如：

```
backgrounds/../characters/角色.png  → characters/角色.png
./backgrounds/bg1.jpg              → backgrounds/bg1.jpg
```

### Q: 打包后的 ZIP 文件很大，如何减小？

**A**: 可以尝试：

1. **压缩图片资源**：使用工具压缩 PNG/JPEG/WebP 文件
2. **移除未使用的资源**：清理 `assets` 目录中不需要的文件

（当前 asset-packer 使用 deflate 默认压缩，不提供压缩级别参数。）

### Q: 开发时可以使用 ZIP 模式吗？

**A**: 可以，但不推荐。ZIP 模式下修改资源需要重新打包，开发效率较低。建议：
- **开发时**：使用文件系统模式（`asset_source: "fs"`）
- **发布前**：切换到 ZIP 模式测试
- **发布时**：使用 ZIP 模式

---

## 技术细节

### 资源加载流程

1. **请求资源**：渲染系统请求纹理（如背景、立绘）
2. **检查缓存**：在 `TextureCache` 中查找
3. **缓存命中**：直接返回缓存的纹理
4. **缓存未命中**：
   - 通过 `ResourceSource` 读取原始数据（文件系统或 ZIP）
   - 解码图片数据
   - 创建 `Texture2D` 对象
   - 存入缓存
   - 返回纹理

### 缓存驱逐策略

当缓存占用超过显存预算时：

1. 按 **FIFO** 顺序（先插入的先逐出）从队头取条目
2. 逐出该条目并释放显存
3. 重复直到占用低于预算或无可逐出条目

**单纹理超预算**：若队列已空仍超预算（例如单张纹理大于 `texture_cache_size_mb`），会输出警告日志并仍插入该纹理，此时占用可能超过预算。

**建议**：
- 若经常看到 “eviction exhausted” 类警告，可：
  - 增大缓存（`config.json` 中的 `texture_cache_size_mb`）
  - 优化资源（压缩图片、减小尺寸）
  - 减少同时显示的立绘数量

### 资源来源抽象

系统使用 `ResourceSource` trait 和 `LogicalPath` newtype 抽象资源访问：

```rust
// LogicalPath：编译期防止逻辑路径与文件系统路径混用
let path = LogicalPath::new("backgrounds/bg.png");

// ResourceSource trait（所有方法使用 &LogicalPath）
trait ResourceSource {
    fn read(&self, path: &LogicalPath) -> Result<Vec<u8>>;
    fn exists(&self, path: &LogicalPath) -> bool;
    fn backing_path(&self, path: &LogicalPath) -> Option<PathBuf>;
    // ...
}
```

实现（`pub(crate)` 可见性，不对外公开）：
- `FsSource`：从文件系统读取
- `ZipSource`：从 ZIP 文件读取

所有资源访问通过 `ResourceManager` 进行，不直接使用 `FsSource`/`ZipSource`。
需要真实文件系统路径时使用 `ResourceManager::materialize_to_fs()`。

---

## 最佳实践

1. **资源组织**：保持清晰的目录结构，便于管理和打包
2. **路径规范**：使用相对路径，避免硬编码绝对路径
3. **资源优化**：压缩图片资源，减小 ZIP 文件大小
4. **定期验证**：发布前使用 `pack verify` 验证 ZIP 完整性
5. **监控缓存**：发布前可通过日志中的缓存统计观察命中率/驱逐次数再调整（参见 [config 配置说明](config.md)）

---

## 相关文档

- [脚本语法规范](script-syntax.md)
- [Manifest 配置指南](manifest.md)
- [存档格式说明](../engine/reference/save-format.md)
