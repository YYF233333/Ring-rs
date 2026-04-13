---
paths:
  - "host/src/resources/**"
  - "host/src/manifest/**"
  - "host/src/config/**"
  - "host/src/save_manager/**"
---

# 资源与配置（resources）

## 摘要导航

- [resources](docs/engine/architecture/module-summaries/host/resources.md)
- [manifest](docs/engine/architecture/module-summaries/host/manifest.md)
- [config](docs/engine/architecture/module-summaries/host/config.md)
- [save_manager](docs/engine/architecture/module-summaries/host/save-manager.md)
- 资源系统：[resource_management.md](docs/authoring/resources.md)
- 配置指南：[config_guide.md](docs/authoring/config.md)

## 关键不变量

- 资源路径**必须**使用 `&LogicalPath`，禁止裸 `&str` / `String` / `PathBuf` 拼接。
- 所有资源读取**必须**通过 `ResourceManager` 统一入口，禁止子系统自持 `base_path`。
- `FsSource` / `ZipSource` 仅在 `init::create_resource_manager` 内部构造；其他位置需 source 时从 `ResourceManager::source()` 获取。
- Config 校验在加载时完成，运行时持有的 `GameConfig` 是已校验的不可变快照。

## Do / Don't

- **Do** 新增资源类型时先在 manifest 中定义 key，再在 `ResourceManager` 添加加载方法。
- **Do** 路径相关测试使用 `LogicalPath::new()` 构造，验证跨平台一致性。
- **Don't** 用 `config.assets_root.join()` 或手工 `PathBuf` 拼接资源路径。
- **Don't** 在 `init.rs` 之外直接构造 `FsSource` / `ZipSource`。
- **Don't** 在运行时修改 `GameConfig`——需要动态设置用 `UserSettings`。
