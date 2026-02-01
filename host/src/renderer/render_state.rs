//! # RenderState 模块
//!
//! 定义渲染状态，管理当前需要渲染的所有元素。

use std::collections::HashMap;
use vn_runtime::command::Position;

/// 渲染状态
///
/// 存储当前帧需要渲染的所有元素状态。
#[derive(Debug, Clone, Default)]
pub struct RenderState {
    /// 当前背景图片路径
    pub current_background: Option<String>,
    
    /// 可见角色列表（alias -> CharacterSprite）
    pub visible_characters: HashMap<String, CharacterSprite>,
    
    /// 当前对话状态
    pub dialogue: Option<DialogueState>,
    
    /// 当前章节标记（用于显示章节过渡）
    pub chapter_mark: Option<ChapterMarkState>,
    
    /// 当前选择界面状态
    pub choices: Option<ChoicesState>,
}

impl RenderState {
    /// 创建空的渲染状态
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置背景
    pub fn set_background(&mut self, path: String) {
        self.current_background = Some(path);
    }

    /// 清除背景
    pub fn clear_background(&mut self) {
        self.current_background = None;
    }

    /// 显示角色
    pub fn show_character(&mut self, alias: String, texture_path: String, position: Position) {
        let z_order = self.visible_characters.len() as i32;
        self.visible_characters.insert(
            alias,
            CharacterSprite {
                texture_path,
                position,
                alpha: 1.0,
                z_order,
            },
        );
    }

    /// 隐藏角色
    pub fn hide_character(&mut self, alias: &str) {
        self.visible_characters.remove(alias);
    }

    /// 隐藏所有角色
    pub fn hide_all_characters(&mut self) {
        self.visible_characters.clear();
    }

    /// 设置对话
    pub fn set_dialogue(&mut self, speaker: Option<String>, content: String) {
        self.dialogue = Some(DialogueState {
            speaker,
            content: content.clone(),
            visible_chars: content.chars().count(), // 默认显示全部
            is_complete: true,
        });
    }

    /// 开始打字机效果
    pub fn start_typewriter(&mut self, speaker: Option<String>, content: String) {
        self.dialogue = Some(DialogueState {
            speaker,
            content,
            visible_chars: 0,
            is_complete: false,
        });
    }

    /// 推进打字机效果（返回是否完成）
    pub fn advance_typewriter(&mut self) -> bool {
        if let Some(ref mut dialogue) = self.dialogue {
            let total_chars = dialogue.content.chars().count();
            if dialogue.visible_chars < total_chars {
                dialogue.visible_chars += 1;
                dialogue.is_complete = dialogue.visible_chars >= total_chars;
            }
            dialogue.is_complete
        } else {
            true
        }
    }

    /// 完成打字机效果（立即显示全部文本）
    pub fn complete_typewriter(&mut self) {
        if let Some(ref mut dialogue) = self.dialogue {
            dialogue.visible_chars = dialogue.content.chars().count();
            dialogue.is_complete = true;
        }
    }

    /// 清除对话
    pub fn clear_dialogue(&mut self) {
        self.dialogue = None;
    }

    /// 检查对话是否完成
    pub fn is_dialogue_complete(&self) -> bool {
        self.dialogue.as_ref().map_or(true, |d| d.is_complete)
    }

    /// 设置章节标记
    pub fn set_chapter_mark(&mut self, title: String, level: u8) {
        self.chapter_mark = Some(ChapterMarkState {
            title,
            level,
            alpha: 1.0,
            timer: 0.0,
        });
    }

    /// 清除章节标记
    pub fn clear_chapter_mark(&mut self) {
        self.chapter_mark = None;
    }

    /// 设置选择界面
    pub fn set_choices(&mut self, choices: Vec<ChoiceItem>, style: Option<String>) {
        self.choices = Some(ChoicesState {
            choices,
            style,
            selected_index: 0,
            hovered_index: None,
        });
    }

    /// 清除选择界面
    pub fn clear_choices(&mut self) {
        self.choices = None;
    }
}

/// 角色立绘状态
#[derive(Debug, Clone)]
pub struct CharacterSprite {
    /// 纹理路径
    pub texture_path: String,
    /// 位置
    pub position: Position,
    /// 透明度 (0.0 - 1.0)
    pub alpha: f32,
    /// 渲染顺序（越大越靠前）
    pub z_order: i32,
}

/// 对话状态
#[derive(Debug, Clone)]
pub struct DialogueState {
    /// 说话者名称（None 表示旁白）
    pub speaker: Option<String>,
    /// 对话内容
    pub content: String,
    /// 当前可见字符数（用于打字机效果）
    pub visible_chars: usize,
    /// 是否显示完成
    pub is_complete: bool,
}

/// 章节标记状态
#[derive(Debug, Clone)]
pub struct ChapterMarkState {
    /// 章节标题
    pub title: String,
    /// 章节级别
    pub level: u8,
    /// 透明度（用于淡入淡出）
    pub alpha: f32,
    /// 计时器（用于动画）
    pub timer: f32,
}

/// 选择项
#[derive(Debug, Clone)]
pub struct ChoiceItem {
    /// 选项文本
    pub text: String,
    /// 目标标签
    pub target_label: String,
}

/// 选择界面状态
#[derive(Debug, Clone)]
pub struct ChoicesState {
    /// 选项列表
    pub choices: Vec<ChoiceItem>,
    /// 样式
    pub style: Option<String>,
    /// 当前选中索引
    pub selected_index: usize,
    /// 鼠标悬停索引
    pub hovered_index: Option<usize>,
}
