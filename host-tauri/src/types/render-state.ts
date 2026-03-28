export interface RenderState {
  current_background: string | null;
  visible_characters: Readonly<Record<string, CharacterSprite>>;
  dialogue: Readonly<DialogueState> | null;
  chapter_mark: Readonly<ChapterMarkState> | null;
  choices: Readonly<ChoicesState> | null;
  ui_visible: boolean;
  title_card: Readonly<TitleCardState> | null;
  scene_effect: Readonly<SceneEffectState>;
  text_mode: "ADV" | "NVL";
  nvl_entries: readonly NvlEntry[];
  background_transition: Readonly<BackgroundTransition> | null;
  scene_transition: Readonly<SceneTransition> | null;
  cutscene: Readonly<CutsceneState> | null;
  playback_mode: PlaybackMode;
  audio: Readonly<AudioRenderState>;
  active_ui_mode: Readonly<UiModeRequest> | null;
}

export interface UiModeRequest {
  mode: string;
  key: string;
  params: Readonly<Record<string, unknown>>;
}

export interface CharacterSprite {
  texture_path: string;
  position:
    | "Left"
    | "Right"
    | "Center"
    | "NearLeft"
    | "NearRight"
    | "NearMiddle"
    | "FarLeft"
    | "FarRight"
    | "FarMiddle";
  z_order: number;
  fading_out: boolean;
  alpha: number;
  offset_x: number;
  offset_y: number;
  scale_x: number;
  scale_y: number;
  transition_duration: number | null;
  target_alpha: number;
  /** 归一化水平位置 (0–1)，来自 manifest preset */
  pos_x: number;
  /** 归一化垂直位置 (0–1)，来自 manifest preset */
  pos_y: number;
  /** 归一化锚点水平偏移 (0–1)，来自 manifest group config */
  anchor_x: number;
  /** 归一化锚点垂直偏移 (0–1)，来自 manifest group config */
  anchor_y: number;
  /** 合成缩放倍率 (pre_scale × preset.scale) */
  render_scale: number;
}

export interface DialogueState {
  speaker: string | null;
  content: string;
  visible_chars: number;
  is_complete: boolean;
  no_wait: boolean;
  inline_wait: { remaining: number | null } | null;
  effective_cps: { Absolute: number } | { Relative: number } | null;
  inline_effects: readonly InlineEffect[];
}

export interface InlineEffect {
  position: number;
  kind: unknown;
}

export interface NvlEntry {
  speaker: string | null;
  content: string;
  visible_chars: number;
  is_complete: boolean;
}

export interface TitleCardState {
  text: string;
  duration: number;
  elapsed: number;
}

export interface SceneEffectState {
  shake_offset_x: number;
  shake_offset_y: number;
  blur_amount: number;
  dim_level: number;
}

export interface ChapterMarkState {
  title: string;
  level: number;
  alpha: number;
  timer: number;
  phase: "FadeIn" | "Visible" | "FadeOut";
}

export interface ChoicesState {
  choices: readonly ChoiceItem[];
  style: string | null;
  selected_index: number;
  hovered_index: number | null;
}

export interface ChoiceItem {
  text: string;
  target_label: string;
}

export interface BackgroundTransition {
  old_background: string | null;
  new_background: string;
  duration: number;
}

export interface SceneTransition {
  transition_type: SceneTransitionKind;
  phase: SceneTransitionPhaseState;
  duration: number;
  pending_background: string | null;
}

export type SceneTransitionKind =
  | "Fade"
  | "FadeWhite"
  | { Rule: { mask_path: string; reversed: boolean; ramp: number } };

export type SceneTransitionPhaseState = "FadeIn" | "Hold" | "FadeOut" | "Completed";

// ── 视频过场 ─────────────────────────────────────────────────────────────────

export interface CutsceneState {
  video_path: string;
  is_playing: boolean;
}

// ── 播放模式 ─────────────────────────────────────────────────────────────────

export type PlaybackMode = "Normal" | "Auto" | "Skip";

// ── 音频状态 ─────────────────────────────────────────────────────────────────

export interface AudioRenderState {
  bgm: Readonly<BgmState> | null;
  sfx_queue: readonly SfxRequest[];
  /** BGM 过渡信号（一次性），前端根据 bgm 变化推断过渡方式 */
  bgm_transition: Readonly<BgmTransition> | null;
}

export interface BgmTransition {
  /** 过渡时长（秒） */
  duration: number;
}

export interface BgmState {
  path: string;
  looping: boolean;
  /** 最终音量 (0.0–1.0)，已含 duck/mute 计算 */
  volume: number;
}

export interface SfxRequest {
  path: string;
  volume: number;
}

// ── 存档 ─────────────────────────────────────────────────────────────────────

export interface SaveInfo {
  slot: number | null;
  timestamp: string;
  chapter_title: string | null;
  script_id: string;
  play_time_secs: number;
}

// ── 历史记录 ─────────────────────────────────────────────────────────────────

export interface HistoryEntry {
  speaker: string | null;
  text: string;
}

// ── 配置 ─────────────────────────────────────────────────────────────────────

export interface AppConfig {
  name: string | null;
  assets_root: string;
  saves_dir: string;
  manifest_path: string;
  start_script_path: string;
  asset_source: "fs" | "zip";
  zip_path: string | null;
  window: {
    width: number;
    height: number;
    title: string;
    fullscreen: boolean;
  };
  debug: {
    log_level: string | null;
  };
  audio: {
    master_volume: number;
    bgm_volume: number;
    sfx_volume: number;
    muted: boolean;
  };
}
