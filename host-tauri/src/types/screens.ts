/** screens.json 的完整类型定义 */

export interface ScreenDefinitions {
  title: TitleScreenDef;
  ingame_menu: InGameMenuDef;
  quick_menu: QuickMenuDef;
  game_menu: GameMenuDef;
}

export interface TitleScreenDef {
  background: ConditionalAsset[];
  overlay: string;
  buttons: ButtonDef[];
}

export interface InGameMenuDef {
  buttons: ButtonDef[];
}

export interface QuickMenuDef {
  buttons: ButtonDef[];
}

export interface GameMenuDef {
  background: ConditionalAsset[];
  overlay: string;
  nav_buttons: ButtonDef[];
  return_button: ButtonDef;
}

export interface ButtonDef {
  label: string;
  action: string | { start_at_label: string };
  visible?: string;
  confirm?: string;
}

export interface ConditionalAsset {
  when?: string;
  asset: string;
}
