# Getting Started：只做内容（脚本/素材/配置）如何做出并发布一个游戏

本文面向 **内容作者**：你只需要写脚本、准备素材、调整配置，然后本地测试与打包发布；不需要改引擎代码。

> 术语对齐：  
> - **资源根目录**：`assets_root`（默认 `assets/`）  
> - **入口脚本**：`start_script_path`（相对于 `assets_root`）  
> - **开发模式**：资源来自文件系统（`asset_source: "fs"`）  
> - **发布模式**：资源来自 ZIP（`asset_source: "zip"`）

## 你需要准备

- **Rust stable**（本仓库使用 `edition = "2024"`，建议用最新稳定版）
- 能跑 OpenGL 的环境

如果你只想“做内容”，但不想装 Rust：你也可以在未来使用预编译的 Player（本仓库暂以源码工作流为主）。

## 1. 创建你的内容目录（推荐结构）

在 `assets/` 下组织你的游戏内容：

```
assets/
├── scripts/         # 脚本 .md
├── backgrounds/     # 背景图
├── characters/      # 立绘
├── bgm/             # 背景音乐/音效
├── fonts/           # 字体（ttf/otf）
└── manifest.json    # 立绘布局配置（可选，但强烈建议尽早配置）
```

支持格式（当前实现）：

- 图片：PNG/JPEG
- 音频：MP3/WAV/FLAC/OGG

## 2. 配置 `config.json`

按照内容需求修改根目录下的`config.json`，该文件将会被打包至最终发行版。至少需要配置：

- `name`：游戏名（也会影响打包后可执行文件名）
- `start_script_path`：入口脚本路径（相对于 `assets_root`）

示例（开发模式）：

```json
{
  "name": "My VN",
  "assets_root": "assets",
  "start_script_path": "scripts/main.md",
  "asset_source": "fs",
  "window": { "width": 1280, "height": 720, "title": "My VN", "fullscreen": false },
  "debug": { "script_check": true, "log_level": "info" }
}
```

路径规则（重要）：

- `manifest_path`、`default_font`、`start_script_path` 都是 **相对于 `assets_root`**
- 例：`default_font: "fonts/simhei.ttf"` 表示 `assets/fonts/simhei.ttf`

## 3. 写你的第一份脚本（`assets/scripts/main.md`）

脚本是 Markdown（为了在 Typora 等编辑器里能预览图片/音频）：

```markdown
# Prologue

changeBG <img src="../backgrounds/room.jpg" /> with dissolve

show <img src="../characters/alice.png" /> as alice at center with dissolve

Alice："早上好。"
："（这是旁白）"

| 选择 |  |
| --- | --- |
| 去学校 | go_school |
| 继续睡 | sleep |

**go_school**
set $mood = "good"
："你出门了。"
goto **end**

**sleep**
set $mood = "bad"
："你又睡着了。"

**end**
if $mood == "good"
  ："今天会是美好的一天。"
else
  ："你错过了很多。"
endif
```

### 资源路径规则（非常关键）

- **脚本里** `<img src="...">` / `<audio src="...">` 的路径为**相对于脚本文件自身**的相对路径
  - 例：`<img src="../backgrounds/bg1.jpg"/>`
  - 好处：在 Typora 里直接预览、拖拽插入也不会乱
  - 如果使用可视化编辑器（Typora等）开发，请在设置中启用相对路径
- 引擎会用“脚本所在目录”作为 base path 去解析并规范化，最终落到 **相对于 `assets_root`** 的资源路径
  - 例如：`assets/scripts/main.md` 里写 `../backgrounds/room.jpg`  
    最终解析为 `backgrounds/room.jpg`

## 4.（可选但推荐）配置立绘布局 `assets/manifest.json`

不同尺寸的立绘想要构图一致，建议尽早配 `manifest.json`：

- 锚点（anchor）：对齐基准点
- 预缩放（pre_scale）：统一不同立绘尺寸
- 站位预设（preset）：`left/center/right/...` 等位置的坐标与缩放

详见：[manifest_guide.md](manifest_guide.md)。

## 5. 本地运行测试

在仓库根目录运行：

```bash
cargo run
```

首次进入标题界面，点击“开始游戏”会加载 `start_script_path` 指定的脚本。

## 6. 静态检查脚本（不运行也能发现问题）

```bash
cargo script-check
```

或检查单个脚本：

```bash
cargo script-check assets/scripts/main.md
```

会检查：

- 语法错误（解析失败）
- 未定义 label（`goto` / choice 目标）
- 资源引用是否存在（背景/立绘/音频）

## 7. 打包发布（生成可分发的 dist/）

一键生成发行版（推荐）：

```bash
cargo pack release --output-dir dist --zip
```

在dist目录下生成完整发行版并压缩成zip包

输出（典型）：

```
dist/
├── My VN.exe      # 可执行文件（根据 config.json.name 自动命名并清理非法字符）
├── game.zip       # 资源包（包含整个 assets/）
└── config.json    # 已自动改成 ZIP 模式（asset_source=zip, zip_path=game.zip）
My VN.zip          # dist目录打包压缩
```

说明：

- `dist/config.json` 会被自动改成 ZIP 模式；**不会修改**仓库根目录的 `config.json`
- 分发时你只需要把 `dist/` 里的这三个文件交付给玩家（不需要再带 `assets/` 目录）

### 发布前自检（强烈建议）

- **验证 ZIP 完整性**：

```bash
cargo pack verify dist/game.zip --input assets
```

- **验证“只靠 ZIP 能跑”**：运行 `dist/` 下的 exe，并确保旁边有 `game.zip` 与 `config.json`

## 常见问题

### Q：我的脚本里路径写了 `../` 会出问题吗？

不会。引擎会做路径规范化（`../`、`./` 会被正确处理），并保证资源缓存键一致。

### Q：发布时为什么不需要 `assets/`？

发布模式下资源来自 `game.zip`（`asset_source: "zip"`），所有 `assets/` 内容都被打进 ZIP 里了。

