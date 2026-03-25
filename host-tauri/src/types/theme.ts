/** layout.json 中 assets 段和 colors 段的类型定义 */

export interface UiAssets {
  textbox: string;
  namebox: string;
  frame: string;
  main_menu_overlay: string;
  game_menu_overlay: string;
  confirm_overlay: string;
  skip: string;
  notify: string;
  main_summer: string;
  main_winter: string;
  game_menu_bg: string;
  button_idle: string;
  button_hover: string;
  choice_idle: string;
  choice_hover: string;
  slot_idle: string;
  slot_hover: string;
  quick_idle: string;
  quick_hover: string;
  slider_idle_bar: string;
  slider_hover_bar: string;
  slider_idle_thumb: string;
  slider_hover_thumb: string;
  [key: string]: string;
}

export interface UiColors {
  accent: string;
  idle: string;
  hover: string;
  selected: string;
  insensitive: string;
  text: string;
  interface_text: string;
  [key: string]: string;
}

export interface UiAssetsAndColors {
  assets: UiAssets;
  colors: UiColors;
}

/** get_ui_condition_context 返回值 */
export interface UiConditionContext {
  has_continue: boolean;
  persistent: Record<string, unknown>;
}
