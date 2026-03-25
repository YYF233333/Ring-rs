import { readonly, ref } from "vue";
import type { ButtonDef, ConditionalAsset, ScreenDefinitions } from "../types/screens";
import type { UiConditionContext } from "../types/theme";
import { callBackend } from "./useBackend";
import { createLogger } from "./useLogger";

const log = createLogger("screens");

const screens = ref<ScreenDefinitions | null>(null);
const conditionCtx = ref<UiConditionContext>({
  has_continue: false,
  persistent: {},
});

/** 对 screens.json 的条件表达式求值 */
function evaluateCondition(expr: string): boolean {
  const negated = expr.startsWith("!");
  const raw = negated ? expr.slice(1) : expr;

  let result: boolean;
  if (raw === "$has_continue") {
    result = conditionCtx.value.has_continue;
  } else if (raw.startsWith("$persistent.")) {
    const key = raw.slice("$persistent.".length);
    const val = conditionCtx.value.persistent[key];
    result = !!val;
  } else {
    result = true;
  }

  return negated ? !result : result;
}

/** 判断按钮是否可见 */
function isButtonVisible(btn: ButtonDef): boolean {
  if (!btn.visible) return true;
  return evaluateCondition(btn.visible);
}

/** 从条件背景列表中选出当前应使用的 asset key */
function resolveConditionalAsset(list: readonly ConditionalAsset[]): string | null {
  for (const entry of list) {
    if (!entry.when || evaluateCondition(entry.when)) {
      return entry.asset;
    }
  }
  return null;
}

/** 将 action 转为标准化字符串（复合 action 归一化） */
function actionId(action: string | { start_at_label: string }): string {
  if (typeof action === "string") return action;
  return `start_at_label:${action.start_at_label}`;
}

export function useScreens() {
  async function init() {
    try {
      const [defs, ctx] = await Promise.all([
        callBackend<ScreenDefinitions>("get_screen_definitions"),
        callBackend<UiConditionContext>("get_ui_condition_context"),
      ]);
      screens.value = defs;
      conditionCtx.value = ctx;
      log.info("screens config loaded");
    } catch (e) {
      log.warn("screens config unavailable, using defaults", e);
    }
  }

  async function refreshConditions() {
    try {
      conditionCtx.value = await callBackend<UiConditionContext>("get_ui_condition_context");
    } catch {
      // best-effort
    }
  }

  return {
    screens: readonly(screens),
    init,
    refreshConditions,
    isButtonVisible,
    resolveConditionalAsset,
    actionId,
  };
}
