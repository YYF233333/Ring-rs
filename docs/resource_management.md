# 资源管理系统使用指南

本指南介绍如何使用 Ring Engine 的资源动态加载系统，包括按需加载、缓存管理、资源打包等功能。

## 目录

- [概述](#概述)
- [配置资源来源](#配置资源来源)
- [缓存系统](#缓存系统)
- [资源打包](#资源打包)
- [Debug Overlay](#debug-overlay)
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

**完整配置示例**：

```json
{
  "assets_root": "assets",
  "saves_dir": "saves",
  "manifest_path": "manifest.json",
  "start_script_path": "scripts/main.md",
  "asset_source": "zip",
  "zip_path": "game.zip",
  "window": {
    "width": 1280,
    "height": 720,
    "title": "My Visual Novel",
    "fullscreen": false
  },
  "resources": {
    "texture_cache_size_mb": 256
  }
}
```

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

# 指定压缩级别（0-9，默认 6）
cargo run -p asset-packer -- --input assets --output game.zip --level 9

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

# 指定压缩级别（0-9，默认 6）
cargo run -p asset-packer -- --level 9
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
2. **打包资源**：运行 `cargo run -p asset-packer`
3. **更新配置**：修改 `config.json`，设置 `asset_source: "zip"` 和 `zip_path: "game.zip"`
4. **测试发布**：删除或重命名 `assets` 目录，运行游戏验证 ZIP 模式
5. **分发**：分发 `exe` + `game.zip` + `config.json`

### 发布目录结构

```
发布目录/
├── host.exe              # 游戏可执行文件
├── game.zip              # 资源包（包含所有 assets）
└── config.json            # 配置文件
```

**注意**：发布时不需要 `assets/` 目录，所有资源都在 `game.zip` 中。

---

## Debug Overlay

### 启用调试信息

按 **F1** 键切换调试模式，左上角会显示资源缓存统计信息。

### 显示内容

调试信息包括：

1. **基础信息**
   - FPS：当前帧率
   - 模式：当前应用模式（Title/InGame/InGameMenu 等）
   - 角色/背景/对话：当前场景状态

2. **纹理缓存统计**
   - **条目数**：当前缓存的纹理数量
   - **占用/大小**：当前显存占用和缓存大小（MB）
   - **命中率**：缓存命中率（百分比）
     - 🟢 绿色：> 80%（优秀）
     - 🟡 黄色：50-80%（良好）
     - 🔴 红色：< 50%（需要优化）
   - **驱逐次数**：累计被驱逐的资源数量
     - 🟢 绿色：0 次（未发生驱逐）
     - 🟡 黄色：> 0 次（发生过驱逐）

3. **资源来源**
   - 文件系统：显示 "文件系统"
   - ZIP 模式：显示 "ZIP: game.zip"

### 示例输出

```
FPS: 60
模式: InGame
角色: 2 | 背景: true | 对话: true
--- 纹理缓存 ---
条目: 15 | 占用: 45.2MB / 256.0MB
命中率: 92.3% (120/130)
驱逐次数: 3
来源: 文件系统
```

### 使用建议

- **开发时**：定期查看命中率，优化资源使用
- **性能调试**：观察驱逐次数，判断是否需要调整缓存大小
- **发布前**：验证 ZIP 模式下的缓存行为

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

### Q: 如何知道哪些资源被缓存了？

**A**: 使用 Debug Overlay（按 F1）查看缓存统计。目前不提供详细的资源列表，但可以通过命中率和驱逐次数判断缓存效果。

### Q: 资源路径中的 `../` 会被处理吗？

**A**: 会的。系统会自动规范化路径，`../` 和 `./` 会被正确处理。例如：

```
backgrounds/../characters/角色.png  → characters/角色.png
./backgrounds/bg1.jpg              → backgrounds/bg1.jpg
```

### Q: 打包后的 ZIP 文件很大，如何减小？

**A**: 可以尝试：

1. **提高压缩级别**：`cargo run -p asset-packer -- --level 9`
2. **压缩图片资源**：使用工具压缩 PNG/JPEG 文件
3. **移除未使用的资源**：清理 `assets` 目录中不需要的文件

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

当缓存占用超过缓存大小时：

1. 遍历 LRU 队列（最久未使用在前）
2. 跳过被 pin 的资源（当前帧正在使用）
3. 驱逐第一个未 pin 的资源
4. 重复直到占用低于缓存大小

**防止死循环机制**：
- 如果扫描完所有资源后仍无法找到可驱逐的资源（可能全部被 pin），系统会：
  - 输出警告日志，提示显存可能溢出
  - 强制插入新资源（可能导致超出缓存大小）
  - 不会进入死循环

**建议**：
- 如果经常看到警告，考虑：
  - 增加缓存大小（`config.json` 中的 `texture_cache_size_mb`）
  - 优化资源大小（压缩图片）
  - 减少同时显示的立绘数量

### 资源来源抽象

系统使用 `ResourceSource` trait 抽象资源访问：

```rust
trait ResourceSource {
    fn read(&self, path: &str) -> Result<Vec<u8>>;
    fn exists(&self, path: &str) -> bool;
    fn full_path(&self, path: &str) -> String;
}
```

实现：
- `FsSource`：从文件系统读取
- `ZipSource`：从 ZIP 文件读取

---

## 最佳实践

1. **资源组织**：保持清晰的目录结构，便于管理和打包
2. **路径规范**：使用相对路径，避免硬编码绝对路径
3. **资源优化**：压缩图片资源，减小 ZIP 文件大小
4. **定期验证**：发布前使用 `pack verify` 验证 ZIP 完整性
5. **监控缓存**：开发时使用 Debug Overlay 监控缓存性能

---

## 相关文档

- [脚本语法规范](script_syntax_spec.md)
- [Manifest 配置指南](manifest_guide.md)
- [存档格式说明](save_format.md)
