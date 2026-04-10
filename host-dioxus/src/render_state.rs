use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use vn_runtime::command::{InlineEffect, InlineEffectKind, Position, TextMode};
use vn_runtime::state::VarValue;

/// 活跃的 UI 模式请求
#[derive(Debug, Clone, Serialize)]
pub struct UiModeRequest {
    pub mode: String,
    pub key: String,
    pub params: HashMap<String, serde_json::Value>,
}

/// 宿主当前屏幕/模式投影。
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

pub fn var_value_to_json(v: &VarValue) -> serde_json::Value {
    match v {
        VarValue::Bool(b) => serde_json::Value::Bool(*b),
        VarValue::Int(i) => serde_json::json!(*i),
        VarValue::Float(f) => serde_json::json!(*f),
        VarValue::String(s) => serde_json::Value::String(s.clone()),
    }
}

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
#[derive(Debug, Clone, Serialize)]
pub struct RenderState {
    pub current_background: Option<String>,
    pub visible_characters: HashMap<String, CharacterSprite>,
    pub dialogue: Option<DialogueState>,
    pub chapter_mark: Option<ChapterMarkState>,
    pub choices: Option<ChoicesState>,
    pub ui_visible: bool,
    pub title_card: Option<TitleCardState>,
    pub scene_effect: SceneEffectState,
    pub text_mode: TextMode,
    pub nvl_entries: Vec<NvlEntry>,
    pub background_transition: Option<BackgroundTransition>,
    pub scene_transition: Option<SceneTransition>,
    pub cutscene: Option<CutsceneState>,
    pub playback_mode: PlaybackMode,
    pub audio: AudioRenderState,
    pub active_ui_mode: Option<UiModeRequest>,
    pub host_screen: HostScreen,
}

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
    pub transition_duration: Option<f32>,
    pub target_alpha: f32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub anchor_x: f32,
    pub anchor_y: f32,
    pub render_scale: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackgroundTransition {
    pub old_background: Option<String>,
    pub new_background: String,
    pub duration: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SceneTransition {
    pub transition_type: SceneTransitionKind,
    pub phase: SceneTransitionPhaseState,
    pub duration: f32,
    pub pending_background: Option<String>,
}

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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum SceneTransitionPhaseState {
    FadeIn,
    Hold,
    FadeOut,
    Completed,
}

#[derive(Debug, Clone, Serialize)]
pub struct DialogueState {
    pub speaker: Option<String>,
    pub content: String,
    pub visible_chars: usize,
    pub is_complete: bool,
    pub inline_effects: Vec<InlineEffect>,
    pub no_wait: bool,
    pub inline_wait: Option<InlineWait>,
    pub effective_cps: Option<EffectiveCps>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InlineWait {
    Click,
    Timed { remaining: f64 },
}

#[derive(Debug, Clone, Serialize)]
pub enum EffectiveCps {
    Absolute(f64),
    Relative(f64),
}

#[derive(Debug, Clone, Serialize)]
pub struct NvlEntry {
    pub speaker: Option<String>,
    pub content: String,
    pub visible_chars: usize,
    pub is_complete: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TitleCardState {
    pub text: String,
    pub duration: f32,
    pub elapsed: f32,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SceneEffectState {
    pub shake_offset_x: f32,
    pub shake_offset_y: f32,
    pub blur_amount: f32,
    pub dim_level: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ChapterMarkPhase {
    FadeIn,
    Visible,
    FadeOut,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChapterMarkState {
    pub title: String,
    pub level: u8,
    pub alpha: f32,
    pub timer: f32,
    pub phase: ChapterMarkPhase,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChoiceItem {
    pub text: String,
    pub target_label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChoicesState {
    pub choices: Vec<ChoiceItem>,
    pub style: Option<String>,
    pub selected_index: usize,
    pub hovered_index: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CutsceneState {
    pub video_path: String,
    pub is_playing: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackMode {
    Normal,
    Auto,
    Skip,
}

#[derive(Debug, Clone, Serialize)]
pub struct AudioRenderState {
    pub bgm: Option<BgmState>,
    pub sfx_queue: Vec<SfxRequest>,
    pub bgm_transition: Option<BgmTransition>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BgmTransition {
    pub duration: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct BgmState {
    pub path: String,
    pub looping: bool,
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

    pub fn set_background(&mut self, path: String) {
        self.current_background = Some(path);
    }

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

    pub fn start_typewriter(
        &mut self,
        speaker: Option<String>,
        content: String,
        inline_effects: Vec<InlineEffect>,
        no_wait: bool,
    ) {
        self.set_dialogue(speaker, content, inline_effects, no_wait);
    }

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

    pub fn has_inline_wait(&self) -> bool {
        self.dialogue
            .as_ref()
            .is_some_and(|d| d.inline_wait.is_some())
    }

    pub fn is_inline_click_wait(&self) -> bool {
        self.dialogue
            .as_ref()
            .is_some_and(|d| matches!(d.inline_wait, Some(InlineWait::Click)))
    }

    pub fn clear_inline_wait(&mut self) {
        if let Some(d) = self.dialogue.as_mut() {
            d.inline_wait = None;
        }
    }

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

    pub fn set_chapter_mark(&mut self, title: String, level: u8) {
        self.chapter_mark = Some(ChapterMarkState {
            title,
            level,
            alpha: 0.0,
            timer: 0.0,
            phase: ChapterMarkPhase::FadeIn,
        });
    }

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
