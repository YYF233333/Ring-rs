//! # RenderState 模块
//!
//! 定义渲染状态，管理当前需要渲染的所有元素。

use std::collections::HashMap;
use vn_runtime::command::Position;

/// 渲染状态
///
/// 存储当前帧需要渲染的所有元素状态。
#[derive(Debug, Clone)]
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

    /// UI 是否可见（用于 changeScene 时隐藏 UI）
    pub ui_visible: bool,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            current_background: None,
            visible_characters: HashMap::new(),
            dialogue: None,
            chapter_mark: None,
            choices: None,
            ui_visible: true,
        }
    }
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
    ///
    /// 创建角色数据和动画状态。初始透明度为 0，需要通过动画系统淡入。
    ///
    /// # 返回
    /// 返回角色的动画状态引用，可用于注册到动画系统
    pub fn show_character(
        &mut self,
        alias: String,
        texture_path: String,
        position: Position,
    ) -> &AnimatableCharacter {
        let z_order = self.visible_characters.len() as i32;

        self.visible_characters.insert(
            alias.clone(),
            CharacterSprite {
                texture_path,
                position,
                z_order,
                fading_out: false,
                anim: AnimatableCharacter::transparent(&alias), // 初始透明，等待淡入
            },
        );

        &self.visible_characters.get(&alias).unwrap().anim
    }

    /// 获取角色的动画状态
    pub fn get_character_anim(&self, alias: &str) -> Option<&AnimatableCharacter> {
        self.visible_characters.get(alias).map(|c| &c.anim)
    }

    /// 获取角色的动画状态（可变）
    pub fn get_character_anim_mut(&mut self, alias: &str) -> Option<&mut AnimatableCharacter> {
        self.visible_characters.get_mut(alias).map(|c| &mut c.anim)
    }

    /// 隐藏角色（立即移除）
    pub fn hide_character(&mut self, alias: &str) {
        self.visible_characters.remove(alias);
    }

    /// 标记角色为淡出状态
    ///
    /// 角色会在动画完成后被 `remove_fading_out_characters` 移除。
    pub fn mark_character_fading_out(&mut self, alias: &str) {
        if let Some(character) = self.visible_characters.get_mut(alias) {
            character.fading_out = true;
        }
    }

    /// 移除所有标记为淡出且动画已完成的角色
    ///
    /// 应在动画系统更新后调用，传入已完成淡出的角色列表。
    pub fn remove_fading_out_characters(&mut self, completed_aliases: &[String]) {
        for alias in completed_aliases {
            if let Some(character) = self.visible_characters.get(alias) {
                if character.fading_out {
                    self.visible_characters.remove(alias);
                }
            }
        }
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

    /// 设置章节标记（覆盖策略：新的直接覆盖旧的）
    ///
    /// 从 FadeIn 阶段开始，alpha = 0，由 update_chapter_mark 驱动动画。
    pub fn set_chapter_mark(&mut self, title: String, level: u8) {
        self.chapter_mark = Some(ChapterMarkState {
            title,
            level,
            alpha: 0.0,
            timer: 0.0,
            phase: ChapterMarkPhase::FadeIn,
        });
    }

    /// 清除章节标记
    pub fn clear_chapter_mark(&mut self) {
        self.chapter_mark = None;
    }

    /// 更新章节标记动画（由每帧 update 调用）
    ///
    /// 返回 true 表示章节标记仍在显示。
    /// 此更新**不受用户快进/点击影响**。
    pub fn update_chapter_mark(&mut self, dt: f32) -> bool {
        let should_clear = if let Some(ref mut mark) = self.chapter_mark {
            mark.timer += dt;
            match mark.phase {
                ChapterMarkPhase::FadeIn => {
                    mark.alpha =
                        (mark.timer / ChapterMarkState::FADE_IN_DURATION).min(1.0);
                    if mark.timer >= ChapterMarkState::FADE_IN_DURATION {
                        mark.phase = ChapterMarkPhase::Visible;
                        mark.timer = 0.0;
                        mark.alpha = 1.0;
                    }
                    false
                }
                ChapterMarkPhase::Visible => {
                    mark.alpha = 1.0;
                    if mark.timer >= ChapterMarkState::VISIBLE_DURATION {
                        mark.phase = ChapterMarkPhase::FadeOut;
                        mark.timer = 0.0;
                    }
                    false
                }
                ChapterMarkPhase::FadeOut => {
                    mark.alpha =
                        1.0 - (mark.timer / ChapterMarkState::FADE_OUT_DURATION).min(1.0);
                    if mark.timer >= ChapterMarkState::FADE_OUT_DURATION {
                        true // 动画完成，需要清除
                    } else {
                        false
                    }
                }
            }
        } else {
            return false;
        };

        if should_clear {
            self.chapter_mark = None;
            false
        } else {
            true
        }
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

use super::character_animation::AnimatableCharacter;

/// 角色立绘状态
///
/// 存储角色立绘的基本信息和动画状态。
#[derive(Debug, Clone)]
pub struct CharacterSprite {
    /// 纹理路径
    pub texture_path: String,
    /// 位置预设
    pub position: Position,
    /// 渲染顺序（越大越靠前）
    pub z_order: i32,
    /// 是否正在淡出（淡出完成后将被移除）
    pub fading_out: bool,
    /// 动画状态（透明度、位置、缩放等）
    pub anim: AnimatableCharacter,
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

/// 章节标记显示阶段
#[derive(Debug, Clone, PartialEq)]
pub enum ChapterMarkPhase {
    /// 淡入阶段
    FadeIn,
    /// 持续显示阶段
    Visible,
    /// 淡出阶段
    FadeOut,
}

/// 章节标记状态
///
/// 阶段 24 重构：非阻塞、固定持续时间、不受快进影响。
/// 在左上角异步显示，固定时间后自动消失。
#[derive(Debug, Clone)]
pub struct ChapterMarkState {
    /// 章节标题
    pub title: String,
    /// 章节级别
    pub level: u8,
    /// 透明度（用于淡入淡出）
    pub alpha: f32,
    /// 当前阶段计时器（秒）
    pub timer: f32,
    /// 当前显示阶段
    pub phase: ChapterMarkPhase,
}

/// ChapterMark 时间常量
impl ChapterMarkState {
    /// 淡入时长（秒）
    const FADE_IN_DURATION: f32 = 0.4;
    /// 持续显示时长（秒）
    const VISIBLE_DURATION: f32 = 3.0;
    /// 淡出时长（秒）
    const FADE_OUT_DURATION: f32 = 0.6;
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

#[cfg(test)]
mod tests {
    use super::*;
    use vn_runtime::command::Position;

    #[test]
    fn test_render_state_default() {
        let state = RenderState::new();
        assert!(state.current_background.is_none());
        assert!(state.visible_characters.is_empty());
        assert!(state.dialogue.is_none());
        assert!(state.chapter_mark.is_none());
        assert!(state.choices.is_none());
        assert!(state.ui_visible);
    }

    #[test]
    fn test_set_background() {
        let mut state = RenderState::new();
        state.set_background("bg.png".to_string());
        assert_eq!(state.current_background, Some("bg.png".to_string()));

        state.clear_background();
        assert!(state.current_background.is_none());
    }

    #[test]
    fn test_show_hide_character() {
        let mut state = RenderState::new();

        // 显示角色
        state.show_character("alice".to_string(), "alice.png".to_string(), Position::Left);
        assert!(state.visible_characters.contains_key("alice"));
        assert_eq!(
            state.visible_characters.get("alice").unwrap().texture_path,
            "alice.png"
        );

        // 隐藏角色
        state.hide_character("alice");
        assert!(!state.visible_characters.contains_key("alice"));
    }

    #[test]
    fn test_hide_all_characters() {
        let mut state = RenderState::new();

        state.show_character("alice".to_string(), "alice.png".to_string(), Position::Left);
        state.show_character("bob".to_string(), "bob.png".to_string(), Position::Right);
        assert_eq!(state.visible_characters.len(), 2);

        state.hide_all_characters();
        assert!(state.visible_characters.is_empty());
    }

    #[test]
    fn test_character_fading_out() {
        let mut state = RenderState::new();

        state.show_character(
            "alice".to_string(),
            "alice.png".to_string(),
            Position::Center,
        );

        // 标记为淡出
        state.mark_character_fading_out("alice");
        assert!(state.visible_characters.get("alice").unwrap().fading_out);

        // 移除淡出完成的角色
        state.remove_fading_out_characters(&["alice".to_string()]);
        assert!(!state.visible_characters.contains_key("alice"));
    }

    #[test]
    fn test_typewriter_effect() {
        let mut state = RenderState::new();

        // 开始打字机效果
        state.start_typewriter(Some("北风".to_string()), "你好世界".to_string());
        let dialogue = state.dialogue.as_ref().unwrap();
        assert_eq!(dialogue.visible_chars, 0);
        assert!(!dialogue.is_complete);
        assert!(!state.is_dialogue_complete());

        // 推进一个字符
        state.advance_typewriter();
        assert_eq!(state.dialogue.as_ref().unwrap().visible_chars, 1);

        // 推进直到完成
        while !state.is_dialogue_complete() {
            state.advance_typewriter();
        }
        assert_eq!(state.dialogue.as_ref().unwrap().visible_chars, 4); // "你好世界" = 4 个字符
        assert!(state.is_dialogue_complete());
    }

    #[test]
    fn test_complete_typewriter() {
        let mut state = RenderState::new();

        state.start_typewriter(None, "测试文本".to_string());
        assert!(!state.is_dialogue_complete());

        // 立即完成
        state.complete_typewriter();
        assert!(state.is_dialogue_complete());
        assert_eq!(state.dialogue.as_ref().unwrap().visible_chars, 4);
    }

    #[test]
    fn test_set_dialogue() {
        let mut state = RenderState::new();

        state.set_dialogue(Some("说话者".to_string()), "内容".to_string());
        let dialogue = state.dialogue.as_ref().unwrap();
        assert_eq!(dialogue.speaker, Some("说话者".to_string()));
        assert_eq!(dialogue.content, "内容");
        assert!(dialogue.is_complete); // set_dialogue 直接显示全部

        state.clear_dialogue();
        assert!(state.dialogue.is_none());
    }

    #[test]
    fn test_chapter_mark() {
        let mut state = RenderState::new();

        state.set_chapter_mark("第一章".to_string(), 1);
        let chapter = state.chapter_mark.as_ref().unwrap();
        assert_eq!(chapter.title, "第一章");
        assert_eq!(chapter.level, 1);
        assert_eq!(chapter.alpha, 0.0); // 从 FadeIn 开始
        assert_eq!(chapter.phase, ChapterMarkPhase::FadeIn);

        state.clear_chapter_mark();
        assert!(state.chapter_mark.is_none());
    }

    #[test]
    fn test_chapter_mark_animation_lifecycle() {
        let mut state = RenderState::new();

        state.set_chapter_mark("第一章".to_string(), 1);
        assert!(state.chapter_mark.is_some());

        // FadeIn 阶段 (FADE_IN_DURATION = 0.4s)
        state.update_chapter_mark(0.2);
        let mark = state.chapter_mark.as_ref().unwrap();
        assert_eq!(mark.phase, ChapterMarkPhase::FadeIn);
        assert!(mark.alpha > 0.0 && mark.alpha < 1.0);

        // 完成 FadeIn → Visible (累计 0.2 + 0.3 = 0.5 > 0.4)
        state.update_chapter_mark(0.3);
        let mark = state.chapter_mark.as_ref().unwrap();
        assert_eq!(mark.phase, ChapterMarkPhase::Visible);
        assert_eq!(mark.alpha, 1.0);

        // Visible 期间保持 (VISIBLE_DURATION = 3.0s)
        state.update_chapter_mark(1.0);
        let mark = state.chapter_mark.as_ref().unwrap();
        assert_eq!(mark.phase, ChapterMarkPhase::Visible);
        assert_eq!(mark.alpha, 1.0);

        // 完成 Visible → FadeOut (需要再过 2.1s 来超过 3.0s)
        state.update_chapter_mark(2.1);
        let mark = state.chapter_mark.as_ref().unwrap();
        assert_eq!(mark.phase, ChapterMarkPhase::FadeOut);

        // FadeOut 阶段开始时 timer 被重置为 0，这个 update 后 timer=0.0 (刚进入)
        // alpha 应该接近 1.0 因为刚进入 FadeOut
        // 继续推进
        state.update_chapter_mark(0.3);
        let mark = state.chapter_mark.as_ref().unwrap();
        assert_eq!(mark.phase, ChapterMarkPhase::FadeOut);
        assert!(mark.alpha < 1.0 && mark.alpha > 0.0);

        // 完成 FadeOut → 自动消失 (FADE_OUT_DURATION = 0.6s)
        state.update_chapter_mark(0.5);
        assert!(state.chapter_mark.is_none());
    }

    #[test]
    fn test_chapter_mark_overlap_replace() {
        let mut state = RenderState::new();

        // 设置第一个
        state.set_chapter_mark("第一章".to_string(), 1);
        state.update_chapter_mark(0.5); // 进入 Visible 阶段

        // 设置第二个（覆盖第一个）
        state.set_chapter_mark("第二章".to_string(), 1);
        let mark = state.chapter_mark.as_ref().unwrap();
        assert_eq!(mark.title, "第二章");
        assert_eq!(mark.phase, ChapterMarkPhase::FadeIn);
        assert_eq!(mark.alpha, 0.0);
    }

    #[test]
    fn test_choices() {
        let mut state = RenderState::new();

        let choices = vec![
            ChoiceItem {
                text: "选项A".to_string(),
                target_label: "labelA".to_string(),
            },
            ChoiceItem {
                text: "选项B".to_string(),
                target_label: "labelB".to_string(),
            },
        ];

        state.set_choices(choices, Some("default".to_string()));
        let choices_state = state.choices.as_ref().unwrap();
        assert_eq!(choices_state.choices.len(), 2);
        assert_eq!(choices_state.style, Some("default".to_string()));
        assert_eq!(choices_state.selected_index, 0);

        state.clear_choices();
        assert!(state.choices.is_none());
    }

    #[test]
    fn test_character_z_order() {
        let mut state = RenderState::new();

        state.show_character("first".to_string(), "first.png".to_string(), Position::Left);
        state.show_character(
            "second".to_string(),
            "second.png".to_string(),
            Position::Right,
        );

        // 后添加的角色 z_order 更大
        assert_eq!(state.visible_characters.get("first").unwrap().z_order, 0);
        assert_eq!(state.visible_characters.get("second").unwrap().z_order, 1);
    }
}
