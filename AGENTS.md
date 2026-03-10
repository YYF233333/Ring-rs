See [CLAUDE.md](CLAUDE.md) for project guidelines and AI agent instructions.



## 注意事项（Cursor）

- PowerShell 不支持 `&&` 作为语句分隔符，用 `;`：
  - 例：`cd F:\Code\Ring-rs; cargo test ...`
- 由于网络原因，工具调用可能失败，如果工具调用多次失败，请停止工作并向用户说明问题，等待用户进一步指示。
- 尽可能少使用🟦等emoji，这些字符可能导致StrReplace工具调用失败。