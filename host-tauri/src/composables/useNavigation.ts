import { readonly, ref } from "vue";

export type Screen = "title" | "ingame" | "save" | "load" | "settings" | "history";

const currentScreen = ref<Screen>("title");
const screenStack = ref<Screen[]>([]);

/** 页面导航状态管理（单例） */
export function useNavigation() {
  function navigateTo(screen: Screen) {
    screenStack.value.push(currentScreen.value);
    currentScreen.value = screen;
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

  return {
    currentScreen: readonly(currentScreen),
    navigateTo,
    goBack,
    resetToTitle,
    resetToIngame,
  };
}
