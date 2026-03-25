import { readonly, ref } from "vue";

export type Screen = "title" | "ingame" | "save" | "load" | "settings" | "history";

/** game_menu 下的子页面（共享 GameMenuFrame 的页面） */
export type GameMenuSubPage = "save" | "load" | "settings" | "history";

const GAME_MENU_PAGES = new Set<Screen>(["save", "load", "settings", "history"]);

const currentScreen = ref<Screen>("title");
const screenStack = ref<Screen[]>([]);

/** 页面导航状态管理（单例） */
export function useNavigation() {
  function navigateTo(screen: Screen) {
    screenStack.value.push(currentScreen.value);
    currentScreen.value = screen;
  }

  /** game_menu 内部替换子页面（不推入 stack） */
  function replaceGameMenuPage(page: GameMenuSubPage) {
    if (GAME_MENU_PAGES.has(currentScreen.value)) {
      currentScreen.value = page;
    } else {
      navigateTo(page);
    }
  }

  function goBack() {
    const prev = screenStack.value.pop();
    if (prev) currentScreen.value = prev;
  }

  function resetToTitle() {
    currentScreen.value = "title";
    screenStack.value = [];
  }

  function resetToIngame() {
    currentScreen.value = "ingame";
    screenStack.value = [];
  }

  /** 当前是否处于 game_menu 框架页面 */
  function isInGameMenu(): boolean {
    return GAME_MENU_PAGES.has(currentScreen.value);
  }

  return {
    currentScreen: readonly(currentScreen),
    navigateTo,
    replaceGameMenuPage,
    goBack,
    resetToTitle,
    resetToIngame,
    isInGameMenu,
  };
}
