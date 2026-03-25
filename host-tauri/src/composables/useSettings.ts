import { ref, readonly } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface UserSettings {
  bgm_volume: number;
  sfx_volume: number;
  text_speed: number;
  auto_delay: number;
  fullscreen: boolean;
}

const DEFAULT_SETTINGS: UserSettings = {
  bgm_volume: 80,
  sfx_volume: 100,
  text_speed: 40,
  auto_delay: 2.0,
  fullscreen: false,
};

const settings = ref<UserSettings>({ ...DEFAULT_SETTINGS });

/** 用户设置管理（单例） */
export function useSettings() {
  async function loadSettings() {
    try {
      const s = await invoke<UserSettings>("get_user_settings");
      settings.value = s;
    } catch {
      settings.value = { ...DEFAULT_SETTINGS };
    }
  }

  async function saveSettings() {
    try {
      await invoke("update_settings", { settings: settings.value });
    } catch (e) {
      console.error("保存设置失败:", e);
    }
  }

  function updateSetting<K extends keyof UserSettings>(
    key: K,
    value: UserSettings[K],
  ) {
    settings.value = { ...settings.value, [key]: value };
  }

  return {
    settings: readonly(settings),
    loadSettings,
    saveSettings,
    updateSetting,
  };
}
