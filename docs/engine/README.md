# 引擎开发文档

这一组文档面向改 `vn-runtime`、`host`、资源系统与 UI 的开发者。

## 从哪里开始

- [架构导航](architecture/README.md)：先看仓库导航地图和模块摘要，决定该读哪个 crate / 模块。
- [工程参考](reference/README.md)：查稳定格式、契约和 capability 说明。
- [UI 文档](ui/README.md)：查 UI 外观和行为相关文档。

## 建议阅读顺序

1. [仓库导航地图](architecture/navigation-map.md)
2. [模块摘要入口](architecture/module-summaries/README.md)
3. [架构约束](../../ARCH.md)
4. [贡献指南](../../CONTRIBUTING.md)
5. [RFC 索引](../../RFCs/README.md)

## 常见任务

- 不知道该改哪里：先看 [仓库导航地图](architecture/navigation-map.md)。
- 想先通过摘要建立心智模型：看 [模块摘要入口](architecture/module-summaries/README.md)。
- 想确认存档和兼容边界：看 [save format](reference/save-format.md)。
- 想确认效果 capability 与回退策略：看 [extension effects capability](reference/extension-effects-capability.md)。
- 想改 UI 外观或页面行为：看 [UI 文档](ui/README.md)。

## 相关入口

- [文档中心](../README.md)
- [测试与调试](../testing/README.md)
- [维护文档](../maintenance/README.md)
