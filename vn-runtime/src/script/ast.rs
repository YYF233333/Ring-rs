//! # AST 模块
//!
//! 定义脚本的抽象语法树（Abstract Syntax Tree）。
//!
//! ## 设计说明
//!
//! AST 是解析器的输出，表示脚本的结构化表示。
//! 执行引擎读取 AST 并产生 Command。

use serde::{Deserialize, Serialize};

use crate::command::{Position, Transition};

/// 选择项（AST 级别）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChoiceOption {
    /// 选项显示文本
    pub text: String,
    /// 跳转目标标签
    pub target_label: String,
}

/// 脚本节点
///
/// 表示脚本中的一个执行单元。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScriptNode {
    /// 章节标记
    ///
    /// 对应 Markdown 标题语法 `# Chapter 1`
    Chapter {
        /// 章节标题
        title: String,
        /// 章节级别（1-6）
        level: u8,
    },

    /// 标签定义
    ///
    /// 对应 `**label_name**` 语法
    Label {
        /// 标签名
        name: String,
    },

    /// 对话
    ///
    /// 对应 `角色名："对话内容"` 语法
    Dialogue {
        /// 说话者名称（None 表示旁白）
        speaker: Option<String>,
        /// 对话内容
        content: String,
    },

    /// 背景切换
    ///
    /// 对应 `changeBG <img> with transition` 语法
    ChangeBG {
        /// 背景图片路径
        path: String,
        /// 过渡效果（可选）
        transition: Option<Transition>,
    },

    /// 场景切换
    ///
    /// 对应 `changeScene <img> with transition` 语法
    ChangeScene {
        /// 场景图片路径
        path: String,
        /// 过渡效果（可选）
        transition: Option<Transition>,
    },

    /// 显示角色
    ///
    /// 对应 `show <img> as alias at position with transition` 或 `show alias at position` 语法
    ShowCharacter {
        /// 立绘图片路径（可选，如果为 None 则使用已绑定的别名）
        path: Option<String>,
        /// 角色别名
        alias: String,
        /// 显示位置
        position: Position,
        /// 过渡效果（可选）
        transition: Option<Transition>,
    },

    /// 隐藏角色
    ///
    /// 对应 `hide alias with transition` 语法
    HideCharacter {
        /// 角色别名
        alias: String,
        /// 过渡效果（可选）
        transition: Option<Transition>,
    },

    /// 选择分支
    ///
    /// 对应 Markdown 表格语法
    Choice {
        /// 选择界面样式（从表头提取）
        style: Option<String>,
        /// 选项列表
        options: Vec<ChoiceOption>,
    },

    /// UI 动画
    ///
    /// 对应 `UIAnim effect` 语法
    UIAnim {
        /// 动画效果
        effect: Transition,
    },

    /// 播放音频
    ///
    /// 对应 `<audio src="..."></audio>` 或 `<audio src="..."></audio> loop` 语法
    PlayAudio {
        /// 音频文件路径
        path: String,
        /// 是否为 BGM（有 loop 标识）
        /// true = BGM（循环播放，同时只有一个）
        /// false = SFX（播放一次）
        is_bgm: bool,
    },

    /// 停止 BGM
    ///
    /// 对应 `stopBGM` 语法
    StopBgm,

    /// 无条件跳转
    ///
    /// 对应 `goto **label**` 语法
    Goto {
        /// 跳转目标标签
        target_label: String,
    },
}

impl ScriptNode {
    /// 判断节点是否会产生等待
    ///
    /// 用于执行引擎判断是否需要暂停执行。
    pub fn causes_wait(&self) -> bool {
        matches!(self, Self::Dialogue { .. } | Self::Choice { .. })
    }

    /// 判断节点是否是跳转目标
    pub fn is_jump_target(&self) -> bool {
        matches!(self, Self::Label { .. })
    }

    /// 如果是标签节点，返回标签名
    pub fn as_label(&self) -> Option<&str> {
        match self {
            Self::Label { name } => Some(name),
            _ => None,
        }
    }
}

/// 解析后的脚本
///
/// 包含节点列表和标签索引，便于跳转。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Script {
    /// 脚本标识符
    pub id: String,
    /// 节点列表
    pub nodes: Vec<ScriptNode>,
    /// 脚本文件所在目录（用于解析相对路径）
    /// 素材路径相对于脚本文件，需要此字段来构建完整路径
    pub base_path: String,
    /// 标签到节点索引的映射
    #[serde(skip)]
    label_index: std::collections::HashMap<String, usize>,
}

impl Script {
    /// 创建新脚本
    ///
    /// # 参数
    /// - `id`: 脚本标识符
    /// - `nodes`: 脚本节点列表
    /// - `base_path`: 脚本文件所在目录，用于解析相对路径
    pub fn new(
        id: impl Into<String>,
        nodes: Vec<ScriptNode>,
        base_path: impl Into<String>,
    ) -> Self {
        let id = id.into();
        let base_path = base_path.into();
        let mut script = Self {
            id,
            nodes,
            base_path,
            label_index: std::collections::HashMap::new(),
        };
        script.build_label_index();
        script
    }

    /// 解析相对于脚本的资源路径，返回完整路径
    ///
    /// 将相对于脚本文件的路径转换为相对于资源根目录的路径
    pub fn resolve_path(&self, relative_path: &str) -> String {
        if relative_path.starts_with('/') || relative_path.starts_with("http") {
            // 绝对路径或 URL，直接返回
            return relative_path.to_string();
        }

        if self.base_path.is_empty() {
            return relative_path.to_string();
        }

        // 拼接脚本目录和相对路径
        format!("{}/{}", self.base_path, relative_path)
    }

    /// 构建标签索引
    fn build_label_index(&mut self) {
        self.label_index.clear();
        for (index, node) in self.nodes.iter().enumerate() {
            if let Some(name) = node.as_label() {
                self.label_index.insert(name.to_string(), index);
            }
        }
    }

    /// 根据标签名查找节点索引
    pub fn find_label(&self, name: &str) -> Option<usize> {
        self.label_index.get(name).copied()
    }

    /// 获取指定索引的节点
    pub fn get_node(&self, index: usize) -> Option<&ScriptNode> {
        self.nodes.get(index)
    }

    /// 节点数量
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_node_causes_wait() {
        let dialogue = ScriptNode::Dialogue {
            speaker: Some("Test".to_string()),
            content: "Hello".to_string(),
        };
        assert!(dialogue.causes_wait());

        let choice = ScriptNode::Choice {
            style: None,
            options: vec![],
        };
        assert!(choice.causes_wait());

        let bg = ScriptNode::ChangeBG {
            path: "bg.png".to_string(),
            transition: None,
        };
        assert!(!bg.causes_wait());
    }

    #[test]
    fn test_script_node_is_jump_target() {
        let label = ScriptNode::Label {
            name: "start".to_string(),
        };
        assert!(label.is_jump_target());

        let dialogue = ScriptNode::Dialogue {
            speaker: None,
            content: "hi".to_string(),
        };
        assert!(!dialogue.is_jump_target());
    }

    #[test]
    fn test_script_label_index() {
        let nodes = vec![
            ScriptNode::Label {
                name: "start".to_string(),
            },
            ScriptNode::Dialogue {
                speaker: None,
                content: "Hello".to_string(),
            },
            ScriptNode::Label {
                name: "end".to_string(),
            },
        ];

        let script = Script::new("test", nodes, "");

        assert_eq!(script.find_label("start"), Some(0));
        assert_eq!(script.find_label("end"), Some(2));
        assert_eq!(script.find_label("nonexistent"), None);
    }

    #[test]
    fn test_script_get_node() {
        let nodes = vec![ScriptNode::Dialogue {
            speaker: None,
            content: "Test".to_string(),
        }];

        let script = Script::new("test", nodes, "");

        assert!(script.get_node(0).is_some());
        assert!(script.get_node(1).is_none());
    }

    #[test]
    fn test_script_is_empty() {
        let s = Script::new("empty", vec![], "");
        assert!(s.is_empty());

        let s = Script::new(
            "not_empty",
            vec![ScriptNode::Dialogue {
                speaker: None,
                content: "x".to_string(),
            }],
            "",
        );
        assert!(!s.is_empty());
    }

    #[test]
    fn test_script_resolve_path() {
        let script = Script::new("test", vec![], "scripts");

        // 相对路径
        assert_eq!(
            script.resolve_path("../bgm/music.mp3"),
            "scripts/../bgm/music.mp3"
        );
        assert_eq!(
            script.resolve_path("images/bg.png"),
            "scripts/images/bg.png"
        );

        // 绝对路径不变
        assert_eq!(
            script.resolve_path("/absolute/path.png"),
            "/absolute/path.png"
        );
        assert_eq!(
            script.resolve_path("http://example.com/img.png"),
            "http://example.com/img.png"
        );

        // 空 base_path
        let script_no_base = Script::new("test", vec![], "");
        assert_eq!(
            script_no_base.resolve_path("images/bg.png"),
            "images/bg.png"
        );
    }
}
