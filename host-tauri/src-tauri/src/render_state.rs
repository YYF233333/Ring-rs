use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use vn_runtime::command::{InlineEffect, InlineEffectKind, Position, TextMode};
use vn_runtime::state::VarValue;

/// 活跃的 UI 模式请求（`requestUI` 命令触发，前端据此展示对应组件）
#[derive(Debug, Clone, Serialize)]
pub struct UiModeRequest {
    pub mode: String,
    pub key: String,
    pub params: HashMap<String, serde_json::Value>,
}

/// 宿主当前屏幕/模式投影。
///
/// 前端页面状态不再是 authority，后端通过该字段声明当前宿主模式。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostScreen {
    Title,
    #[serde(rename = "ingame")]
    InGame,
    InGameMenu,
    Save,
    Load,
    Settings,
    History,
}

impl HostScreen {
    pub fn allows_progression(&self) -> bool {
        matches!(self, Self::InGame)
    }
}

/// 将 `VarValue` 转换为 JSON 友好的 `serde_json::Value`
pub fn var_value_to_json(v: &VarValue) -> serde_json::Value {
    match v {
        VarValue::Bool(b) => serde_json::Value::Bool(*b),
        VarValue::Int(i) => serde_json::json!(*i),
        VarValue::Float(f) => serde_json::json!(*f),
        VarValue::String(s) => serde_json::Value::String(s.clone()),
    }
}

/// 将 JSON `serde_json::Value` 转换为 `VarValue`
pub fn json_to_var_value(v: &serde_json::Value) -> VarValue {
    match v {
        serde_json::Value::Bool(b) => VarValue::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                VarValue::Int(i)
            } else {
                VarValue::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => VarValue::String(s.clone()),
        _ => VarValue::String(v.to_string()),
    }
}

/// 将 Position 枚举转换为 manifest 预设名称
pub(crate) fn position_to_preset_name(position: Position) -> &'static str {
    match position {
        Position::Left => "left",
        Position::NearLeft => "nearleft",
        Position::FarLeft => "farleft",
        Position::Center => "center",
        Position::NearMiddle => "nearmiddle",
        Position::FarMiddle => "farmiddle",
        Position::Right => "right",
        Position::NearRight => "nearright",
        Position::FarRight => "farright",
    }
}

/// 当前帧的完整渲染状态
///
/// 通过 Tauri IPC 序列化后推送给 Vue 前端。
#[derive(Debug, Clone, Serialize)]
pub struct RenderState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_background: Option<String>,
    pub visible_characters: HashMap<String, CharacterSprite>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dialogue: Option<DialogueState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_mark: Option<ChapterMarkState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub choices: Option<ChoicesState>,
    pub ui_visible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_card: Option<TitleCardState>,
    pub scene_effect: SceneEffectState,
    pub text_mode: TextMode,
    pub nvl_entries: Vec<NvlEntry>,
    /// 背景过渡状态（dissolve 时有值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_transition: Option<BackgroundTransition>,
    /// 场景过渡状态（changeScene fade/fadewhite/rule 时有值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scene_transition: Option<SceneTransition>,
    /// 视频过场状态
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cutscene: Option<CutsceneState>,
    /// 当前播放模式
    pub playback_mode: PlaybackMode,
    /// 音频声明式状态
    pub audio: AudioRenderState,
    /// 活跃的 UI 模式请求（`requestUI` 触发时有值，前端据此展示对应 UI 组件）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_ui_mode: Option<UiModeRequest>,
    /// 后端 authoritative 的宿主模式投影
    pub host_screen: HostScreen,
}

/// 角色立绘在场景中的显示状态
#[derive(Debug, Clone, Serialize)]
pub struct CharacterSprite {
    pub texture_path: String,
    pub position: Position,
    pub z_order: i32,
    pub fading_out: bool,
    pub alpha: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    /// 过渡时长（秒），有值时前端用 CSS transition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transition_duration: Option<f32>,
    /// 目标 alpha（前端用 CSS transition 动画到此值）
    pub target_alpha: f32,
    /// 归一化水平位置 (0–1)，来自 manifest preset
    pub pos_x: f32,
    /// 归一化垂直位置 (0–1)，来自 manifest preset
    pub pos_y: f32,
    /// 归一化锚点水平偏移 (0–1)，来自 manifest group config
    pub anchor_x: f32,
    /// 归一化锚点垂直偏移 (0–1)，来自 manifest group config
    pub anchor_y: f32,
    /// 合成缩放倍率 (pre_scale × preset.scale)
    pub render_scale: f32,
}

/// 背景 dissolve 过渡（声明式：前端用 duration 设 CSS transition）
#[derive(Debug, Clone, Serialize)]
pub struct BackgroundTransition {
    /// 旧背景路径
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_background: Option<String>,
    /// 新背景路径
    pub new_background: String,
    /// 过渡时长（秒）
    pub duration: f32,
}

/// 场景遮罩过渡（声明式：前端根据 phase + duration 设 CSS transition）
#[derive(Debug, Clone, Serialize)]
pub struct SceneTransition {
    /// 过渡类型
    pub transition_type: SceneTransitionKind,
    /// 当前阶段（后端按 duration 计时推进）
    pub phase: SceneTransitionPhaseState,
    /// 每阶段时长（秒）
    pub duration: f32,
    /// 待切换背景
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_background: Option<String>,
}

/// 场景过渡效果类型
#[derive(Debug, Clone, Serialize)]
pub enum SceneTransitionKind {
    Fade,
    FadeWhite,
    Rule {
        mask_path: String,
        reversed: bool,
        ramp: f32,
    },
}

/// 场景过渡阶段
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum SceneTransitionPhaseState {
    FadeIn,
    Hold,
    FadeOut,
    Completed,
}

/// 当前对话框的打字机状态
#[derive(Debug, Clone, Serialize)]
pub struct DialogueState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speaker: Option<String>,
    pub content: String,
    pub visible_chars: usize,
    pub is_complete: bool,
    pub inline_effects: Vec<InlineEffect>,
    pub no_wait: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_wait: Option<InlineWait>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_cps: Option<EffectiveCps>,
}

/// 内联等待标记（`{wait}` / `{wait Ns}`）
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InlineWait {
    Click,
    Timed { remaining: f64 },
}

/// 当前生效的文字速度覆盖
#[derive(Debug, Clone, Serialize)]
pub enum EffectiveCps {
    Absolute(f64),
    Relative(f64),
}

/// NVL 模式下的累积文本条目
#[derive(Debug, Clone, Serialize)]
pub struct NvlEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speaker: Option<String>,
    pub content: String,
    pub visible_chars: usize,
    pub is_complete: bool,
}

/// 章节字卡（TitleCard）显示状态
#[derive(Debug, Clone, Serialize)]
pub struct TitleCardState {
    pub text: String,
    pub duration: f32,
    pub elapsed: f32,
}

/// 场景效果（shake / blur / dim）的瞬时值
#[derive(Debug, Clone, Default, Serialize)]
pub struct SceneEffectState {
    pub shake_offset_x: f32,
    pub shake_offset_y: f32,
    pub blur_amount: f32,
    pub dim_level: f32,
}

/// 章节标记的淡入淡出阶段
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ChapterMarkPhase {
    FadeIn,
    Visible,
    FadeOut,
}

/// 章节标记（`# title`）的显示状态
#[derive(Debug, Clone, Serialize)]
pub struct ChapterMarkState {
    pub title: String,
    pub level: u8,
    pub alpha: f32,
    pub timer: f32,
    pub phase: ChapterMarkPhase,
}

/// 选择界面中的单个选项
#[derive(Debug, Clone, Serialize)]
pub struct ChoiceItem {
    pub text: String,
    pub target_label: String,
}

/// 选择界面状态
#[derive(Debug, Clone, Serialize)]
pub struct ChoicesState {
    pub choices: Vec<ChoiceItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    pub selected_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hovered_index: Option<usize>,
}

/// 视频过场状态
#[derive(Debug, Clone, Serialize)]
pub struct CutsceneState {
    pub video_path: String,
    pub is_playing: bool,
}

/// 播放模式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackMode {
    Normal,
    Auto,
    Skip,
}

/// 音频声明式状态——后端描述"应该播什么"，前端负责实际播放
#[derive(Debug, Clone, Serialize)]
pub struct AudioRenderState {
    /// 当前应播放的 BGM（None 表示静音）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bgm: Option<BgmState>,
    /// 本帧需要播放的一次性音效（前端播放后忽略，下帧清空）
    pub sfx_queue: Vec<SfxRequest>,
    /// BGM 过渡信号（一次性，前端消费后下帧为 None）
    ///
    /// 前端根据 `bgm` 字段的变化类型推断过渡方式：
    /// - Some → None：fade out
    /// - None → Some：fade in
    /// - Some(A) → Some(B)：crossfade
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bgm_transition: Option<BgmTransition>,
}

/// BGM 过渡参数（一次性信号）
#[derive(Debug, Clone, Serialize)]
pub struct BgmTransition {
    /// 过渡时长（秒）
    pub duration: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct BgmState {
    pub path: String,
    pub looping: bool,
    /// 最终音量 (0.0–1.0)，已含 duck/mute 计算
    pub volume: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SfxRequest {
    pub path: String,
    pub volume: f32,
}

impl AudioRenderState {
    pub fn silent() -> Self {
        Self {
            bgm: None,
            sfx_queue: Vec::new(),
            bgm_transition: None,
        }
    }
}

const CHAPTER_MARK_FADE_IN: f32 = 0.5;
const CHAPTER_MARK_VISIBLE: f32 = 2.0;
const CHAPTER_MARK_FADE_OUT: f32 = 0.5;

impl RenderState {
    /// 创建空的初始渲染状态
    pub fn new() -> Self {
        Self {
            current_background: None,
            visible_characters: HashMap::new(),
            dialogue: None,
            chapter_mark: None,
            choices: None,
            ui_visible: true,
            title_card: None,
            scene_effect: SceneEffectState::default(),
            text_mode: TextMode::ADV,
            nvl_entries: Vec::new(),
            background_transition: None,
            scene_transition: None,
            cutscene: None,
            playback_mode: PlaybackMode::Normal,
            audio: AudioRenderState::silent(),
            active_ui_mode: None,
            host_screen: HostScreen::Title,
        }
    }

    // ── 背景 ──

    pub fn set_background(&mut self, path: String) {
        self.current_background = Some(path);
    }

    // ── 角色 ──

    /// 添加角色立绘
    pub fn show_character(
        &mut self,
        alias: String,
        texture_path: String,
        position: Position,
        manifest: &crate::manifest::Manifest,
    ) {
        let preset_name = position_to_preset_name(position);
        let preset = manifest.get_preset(preset_name);
        let group = manifest.get_group_config(&texture_path);
        let sprite = CharacterSprite {
            texture_path,
            position,
            z_order: 0,
            fading_out: false,
            alpha: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            transition_duration: None,
            target_alpha: 1.0,
            pos_x: preset.x,
            pos_y: preset.y,
            anchor_x: group.anchor.x,
            anchor_y: group.anchor.y,
            render_scale: group.pre_scale * preset.scale,
        };
        self.visible_characters.insert(alias, sprite);
    }

    pub fn hide_character(&mut self, alias: &str) {
        self.visible_characters.remove(alias);
    }

    pub fn hide_all_characters(&mut self) {
        self.visible_characters.clear();
    }

    // ── 对话 / 打字机 ──

    pub fn set_dialogue(
        &mut self,
        speaker: Option<String>,
        content: String,
        inline_effects: Vec<InlineEffect>,
        no_wait: bool,
    ) {
        self.dialogue = Some(DialogueState {
            speaker,
            content,
            visible_chars: 0,
            is_complete: false,
            inline_effects,
            no_wait,
            inline_wait: None,
            effective_cps: None,
        });
    }

    /// 开始打字机效果——visible_chars 从 0 开始
    pub fn start_typewriter(
        &mut self,
        speaker: Option<String>,
        content: String,
        inline_effects: Vec<InlineEffect>,
        no_wait: bool,
    ) {
        self.set_dialogue(speaker, content, inline_effects, no_wait);
    }

    /// 推进一个可见字符，触发命中的 inline effect
    ///
    /// 返回 `true` 表示文本已全部显示。
    pub fn advance_typewriter(&mut self) -> bool {
        let Some(d) = self.dialogue.as_mut() else {
            return true;
        };
        if d.is_complete {
            return true;
        }

        d.visible_chars += 1;

        for effect in &d.inline_effects {
            if effect.position == d.visible_chars {
                match &effect.kind {
                    InlineEffectKind::Wait(duration) => {
                        d.inline_wait = Some(match *duration {
                            Some(t) => InlineWait::Timed { remaining: t },
                            None => InlineWait::Click,
                        });
                    }
                    InlineEffectKind::SetCpsAbsolute(cps) => {
                        d.effective_cps = Some(EffectiveCps::Absolute(*cps));
                    }
                    InlineEffectKind::SetCpsRelative(multiplier) => {
                        d.effective_cps = Some(EffectiveCps::Relative(*multiplier));
                    }
                    InlineEffectKind::ResetCps => {
                        d.effective_cps = None;
                    }
                }
            }
        }

        let total = d.content.chars().count();
        if d.visible_chars >= total {
            d.is_complete = true;
        }
        let complete = d.is_complete;
        let visible = d.visible_chars;

        if let Some(entry) = self.nvl_entries.last_mut() {
            entry.visible_chars = visible;
            entry.is_complete = complete;
        }

        complete
    }

    pub fn complete_typewriter(&mut self) {
        if let Some(d) = self.dialogue.as_mut() {
            d.visible_chars = d.content.chars().count();
            d.is_complete = true;
            d.inline_wait = None;
            d.effective_cps = None;
        }
        if let Some(entry) = self.nvl_entries.last_mut() {
            entry.visible_chars = entry.content.chars().count();
            entry.is_complete = true;
        }
    }

    /// 追加文本到当前对话（ExtendText 语义）
    pub fn extend_dialogue(
        &mut self,
        content: String,
        inline_effects: Vec<InlineEffect>,
        no_wait: bool,
    ) {
        if let Some(d) = self.dialogue.as_mut() {
            let old_len = d.content.chars().count();
            d.content.push_str(&content);
            let shifted: Vec<InlineEffect> = inline_effects
                .into_iter()
                .map(|mut e| {
                    e.position += old_len;
                    e
                })
                .collect();
            d.inline_effects.extend(shifted);
            d.is_complete = false;
            d.no_wait = no_wait;
        }
        if let Some(entry) = self.nvl_entries.last_mut() {
            entry.content.push_str(&content);
            entry.is_complete = false;
        }
    }

    pub fn clear_dialogue(&mut self) {
        self.dialogue = None;
    }

    pub fn is_dialogue_complete(&self) -> bool {
        self.dialogue.as_ref().is_none_or(|d| d.is_complete)
    }

    // ── Inline wait ──

    pub fn has_inline_wait(&self) -> bool {
        self.dialogue
            .as_ref()
            .is_some_and(|d| d.inline_wait.is_some())
    }

    /// 是否为点击等待型的 inline wait（`{wait}` 无时间参数）
    pub fn is_inline_click_wait(&self) -> bool {
        self.dialogue.as_ref().is_some_and(|d| {
            matches!(d.inline_wait, Some(InlineWait::Click))
        })
    }

    pub fn clear_inline_wait(&mut self) {
        if let Some(d) = self.dialogue.as_mut() {
            d.inline_wait = None;
        }
    }

    /// 更新定时型 inline wait，返回 `true` 表示等待已结束
    pub fn update_inline_wait(&mut self, dt: f64) -> bool {
        let Some(d) = self.dialogue.as_mut() else {
            return true;
        };
        match &mut d.inline_wait {
            Some(InlineWait::Timed { remaining }) => {
                *remaining -= dt;
                if *remaining <= 0.0 {
                    d.inline_wait = None;
                    true
                } else {
                    false
                }
            }
            Some(InlineWait::Click) => false,
            None => true,
        }
    }

    /// 获取当前生效的文字速度（字符/秒），用于打字机推进
    pub fn effective_text_speed(&self, base_speed: f32) -> f32 {
        let Some(d) = self.dialogue.as_ref() else {
            return base_speed;
        };
        match &d.effective_cps {
            None => base_speed,
            Some(EffectiveCps::Absolute(cps)) => *cps as f32,
            Some(EffectiveCps::Relative(mul)) => base_speed * *mul as f32,
        }
    }

    // ── 章节标记 ──

    pub fn set_chapter_mark(&mut self, title: String, level: u8) {
        self.chapter_mark = Some(ChapterMarkState {
            title,
            level,
            alpha: 0.0,
            timer: 0.0,
            phase: ChapterMarkPhase::FadeIn,
        });
    }

    /// 推进章节标记动画，返回 `true` 表示动画结束
    pub fn update_chapter_mark(&mut self, dt: f32) -> bool {
        let Some(cm) = self.chapter_mark.as_mut() else {
            return true;
        };
        cm.timer += dt;
        match cm.phase {
            ChapterMarkPhase::FadeIn => {
                cm.alpha = (cm.timer / CHAPTER_MARK_FADE_IN).min(1.0);
                if cm.timer >= CHAPTER_MARK_FADE_IN {
                    cm.phase = ChapterMarkPhase::Visible;
                    cm.timer = 0.0;
                    cm.alpha = 1.0;
                }
                false
            }
            ChapterMarkPhase::Visible => {
                if cm.timer >= CHAPTER_MARK_VISIBLE {
                    cm.phase = ChapterMarkPhase::FadeOut;
                    cm.timer = 0.0;
                }
                false
            }
            ChapterMarkPhase::FadeOut => {
                cm.alpha = 1.0 - (cm.timer / CHAPTER_MARK_FADE_OUT).min(1.0);
                if cm.timer >= CHAPTER_MARK_FADE_OUT {
                    self.chapter_mark = None;
                    return true;
                }
                false
            }
        }
    }

    // ── 选择 ──

    pub fn set_choices(&mut self, choices: Vec<ChoiceItem>, style: Option<String>) {
        self.choices = Some(ChoicesState {
            choices,
            style,
            selected_index: 0,
            hovered_index: None,
        });
    }

    pub fn clear_choices(&mut self) {
        self.choices = None;
    }
}

impl Default for RenderState {
    fn default() -> Self {
        Self::new()
    }
}
