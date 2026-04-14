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

#[derive(Debug, Clone, PartialEq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SceneTransition {
    pub transition_type: SceneTransitionKind,
    pub phase: SceneTransitionPhaseState,
    pub duration: f32,
    pub pending_background: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChoiceItem {
    pub text: String,
    pub target_label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChoicesState {
    pub choices: Vec<ChoiceItem>,
    pub style: Option<String>,
    pub selected_index: usize,
    pub hovered_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AudioRenderState {
    pub bgm: Option<BgmState>,
    pub sfx_queue: Vec<SfxRequest>,
    pub bgm_transition: Option<BgmTransition>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BgmTransition {
    pub duration: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BgmState {
    pub path: String,
    pub looping: bool,
    pub volume: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
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

#[cfg(test)]
mod tests {
    use vn_runtime::command::{InlineEffect, InlineEffectKind};
    use vn_runtime::state::VarValue;

    use super::*;

    fn make_typewriter(content: &str) -> RenderState {
        let mut rs = RenderState::new();
        rs.start_typewriter(None, content.to_string(), vec![], false);
        rs
    }

    // ── advance_typewriter ─────────────────────────────────────────────────────

    #[test]
    fn advance_typewriter_increments_visible_chars() {
        let mut rs = make_typewriter("Hello");
        let complete = rs.advance_typewriter();
        assert!(!complete);
        assert_eq!(rs.dialogue.as_ref().unwrap().visible_chars, 1);
    }

    #[test]
    fn advance_typewriter_returns_true_when_all_chars_visible() {
        let mut rs = make_typewriter("Hi");
        rs.advance_typewriter(); // 'H'
        let complete = rs.advance_typewriter(); // 'i' — last char
        assert!(complete);
        assert!(rs.dialogue.as_ref().unwrap().is_complete);
    }

    #[test]
    fn advance_typewriter_fires_click_wait_at_position() {
        let effects = vec![InlineEffect {
            position: 2,
            kind: InlineEffectKind::Wait(None),
        }];
        let mut rs = RenderState::new();
        rs.start_typewriter(None, "AB".to_string(), effects, false);
        rs.advance_typewriter(); // pos 1
        rs.advance_typewriter(); // pos 2 → fires Wait(None)
        assert!(rs.is_inline_click_wait());
    }

    #[test]
    fn advance_typewriter_fires_timed_wait_at_position() {
        let effects = vec![InlineEffect {
            position: 1,
            kind: InlineEffectKind::Wait(Some(1.5)),
        }];
        let mut rs = RenderState::new();
        rs.start_typewriter(None, "A".to_string(), effects, false);
        rs.advance_typewriter(); // pos 1 → fires Wait(Some(1.5))
        let d = rs.dialogue.as_ref().unwrap();
        assert!(matches!(d.inline_wait, Some(InlineWait::Timed { remaining }) if remaining > 0.0));
    }

    #[test]
    fn advance_typewriter_fires_set_cps_absolute() {
        let effects = vec![InlineEffect {
            position: 1,
            kind: InlineEffectKind::SetCpsAbsolute(20.0),
        }];
        let mut rs = RenderState::new();
        rs.start_typewriter(None, "A".to_string(), effects, false);
        rs.advance_typewriter();
        assert!(matches!(rs.dialogue.as_ref().unwrap().effective_cps,
                Some(EffectiveCps::Absolute(v)) if (v - 20.0).abs() < f64::EPSILON));
    }

    #[test]
    fn advance_typewriter_fires_set_cps_relative() {
        let effects = vec![InlineEffect {
            position: 1,
            kind: InlineEffectKind::SetCpsRelative(0.5),
        }];
        let mut rs = RenderState::new();
        rs.start_typewriter(None, "A".to_string(), effects, false);
        rs.advance_typewriter();
        assert!(matches!(rs.dialogue.as_ref().unwrap().effective_cps,
                Some(EffectiveCps::Relative(v)) if (v - 0.5).abs() < f64::EPSILON));
    }

    #[test]
    fn advance_typewriter_resets_cps() {
        let effects = vec![
            InlineEffect {
                position: 1,
                kind: InlineEffectKind::SetCpsAbsolute(10.0),
            },
            InlineEffect {
                position: 2,
                kind: InlineEffectKind::ResetCps,
            },
        ];
        let mut rs = RenderState::new();
        rs.start_typewriter(None, "AB".to_string(), effects, false);
        rs.advance_typewriter(); // sets Absolute
        rs.advance_typewriter(); // resets
        assert!(rs.dialogue.as_ref().unwrap().effective_cps.is_none());
    }

    // ── complete_typewriter ────────────────────────────────────────────────────

    #[test]
    fn complete_typewriter_skips_to_end_and_clears_wait() {
        let effects = vec![InlineEffect {
            position: 1,
            kind: InlineEffectKind::Wait(None),
        }];
        let mut rs = RenderState::new();
        rs.start_typewriter(None, "Hello World".to_string(), effects, false);
        rs.advance_typewriter(); // triggers click wait at pos 1
        rs.complete_typewriter();
        let d = rs.dialogue.as_ref().unwrap();
        assert!(d.is_complete);
        assert_eq!(d.visible_chars, "Hello World".chars().count());
        assert!(d.inline_wait.is_none());
        assert!(d.effective_cps.is_none());
    }

    // ── extend_dialogue ────────────────────────────────────────────────────────

    #[test]
    fn extend_dialogue_appends_content_and_shifts_effect_positions() {
        let mut rs = make_typewriter("Hello");
        // advance to completion
        while !rs.advance_typewriter() {}
        let ext_effects = vec![InlineEffect {
            position: 1,
            kind: InlineEffectKind::Wait(None),
        }];
        rs.extend_dialogue(" World".to_string(), ext_effects, false);
        let d = rs.dialogue.as_ref().unwrap();
        assert_eq!(d.content, "Hello World");
        assert!(!d.is_complete);
        // Effect position shifted by original length ("Hello" = 5 chars): 5 + 1 = 6
        assert_eq!(d.inline_effects[0].position, 6);
    }

    // ── effective_text_speed ───────────────────────────────────────────────────

    #[test]
    fn effective_text_speed_returns_base_when_no_override() {
        let rs = make_typewriter("test");
        assert_eq!(rs.effective_text_speed(30.0), 30.0);
    }

    #[test]
    fn effective_text_speed_returns_absolute_cps() {
        let effects = vec![InlineEffect {
            position: 1,
            kind: InlineEffectKind::SetCpsAbsolute(100.0),
        }];
        let mut rs = RenderState::new();
        rs.start_typewriter(None, "A".to_string(), effects, false);
        rs.advance_typewriter();
        assert_eq!(rs.effective_text_speed(30.0), 100.0);
    }

    #[test]
    fn effective_text_speed_returns_relative_cps() {
        let effects = vec![InlineEffect {
            position: 1,
            kind: InlineEffectKind::SetCpsRelative(2.0),
        }];
        let mut rs = RenderState::new();
        rs.start_typewriter(None, "A".to_string(), effects, false);
        rs.advance_typewriter();
        assert_eq!(rs.effective_text_speed(30.0), 60.0);
    }

    // ── update_chapter_mark ────────────────────────────────────────────────────

    #[test]
    fn chapter_mark_transitions_through_three_phases() {
        let mut rs = RenderState::new();
        rs.set_chapter_mark("Chapter 1".to_string(), 1);

        // FadeIn phase (0.5 s threshold)
        assert!(!rs.update_chapter_mark(0.1));
        assert_eq!(
            rs.chapter_mark.as_ref().unwrap().phase,
            ChapterMarkPhase::FadeIn
        );

        // Advance past FadeIn → transitions to Visible
        assert!(!rs.update_chapter_mark(0.5));
        assert_eq!(
            rs.chapter_mark.as_ref().unwrap().phase,
            ChapterMarkPhase::Visible
        );
        assert_eq!(rs.chapter_mark.as_ref().unwrap().alpha, 1.0);

        // Advance past Visible (2.0 s) → transitions to FadeOut
        assert!(!rs.update_chapter_mark(2.1));
        assert_eq!(
            rs.chapter_mark.as_ref().unwrap().phase,
            ChapterMarkPhase::FadeOut
        );

        // Advance past FadeOut (0.5 s) → returns true, chapter_mark removed
        assert!(rs.update_chapter_mark(0.5));
        assert!(rs.chapter_mark.is_none());
    }

    // ── update_inline_wait ────────────────────────────────────────────────────

    #[test]
    fn update_inline_wait_timed_counts_down_and_clears() {
        let effects = vec![InlineEffect {
            position: 1,
            kind: InlineEffectKind::Wait(Some(1.0)),
        }];
        let mut rs = RenderState::new();
        rs.start_typewriter(None, "A".to_string(), effects, false);
        rs.advance_typewriter(); // triggers timed wait 1.0 s

        assert!(!rs.update_inline_wait(0.4));
        assert!(!rs.update_inline_wait(0.4));
        // Remaining ≈ 0.2; one more step of 0.3 pushes it below 0
        assert!(rs.update_inline_wait(0.3));
        assert!(rs.dialogue.as_ref().unwrap().inline_wait.is_none());
    }

    #[test]
    fn update_inline_wait_click_type_never_auto_resolves() {
        let effects = vec![InlineEffect {
            position: 1,
            kind: InlineEffectKind::Wait(None),
        }];
        let mut rs = RenderState::new();
        rs.start_typewriter(None, "A".to_string(), effects, false);
        rs.advance_typewriter(); // triggers click wait

        // Large dt should NOT resolve a click wait
        assert!(!rs.update_inline_wait(999.0));
        assert!(rs.is_inline_click_wait());
    }

    // ── var_value ↔ json conversion ───────────────────────────────────────────

    #[test]
    fn var_value_to_json_bool_roundtrip() {
        let v = VarValue::Bool(true);
        let json = var_value_to_json(&v);
        assert_eq!(json_to_var_value(&json), VarValue::Bool(true));
    }

    #[test]
    fn var_value_to_json_int_roundtrip() {
        let v = VarValue::Int(42);
        let json = var_value_to_json(&v);
        assert_eq!(json_to_var_value(&json), VarValue::Int(42));
    }

    #[test]
    fn var_value_to_json_string_roundtrip() {
        let v = VarValue::String("hello".into());
        let json = var_value_to_json(&v);
        assert_eq!(json_to_var_value(&json), VarValue::String("hello".into()));
    }
}
