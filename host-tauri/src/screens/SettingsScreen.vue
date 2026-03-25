<script setup lang="ts">
import { onMounted } from "vue";
import { useSettings } from "../composables/useSettings";

const emit = defineEmits<{
  back: [];
}>();

const { settings, loadSettings, saveSettings, updateSetting } = useSettings();

onMounted(() => {
  loadSettings();
});

function onSlider(key: "bgm_volume" | "sfx_volume" | "text_speed", e: Event) {
  const val = parseFloat((e.target as HTMLInputElement).value);
  updateSetting(key, val);
}

function onAutoDelay(e: Event) {
  const val = parseFloat((e.target as HTMLInputElement).value);
  updateSetting("auto_delay", val);
}

function toggleFullscreen() {
  updateSetting("fullscreen", !settings.value.fullscreen);
}

async function onBack() {
  await saveSettings();
  emit("back");
}
</script>

<template>
  <div class="settings-screen">
    <header class="settings-header">
      <h2 class="settings-title">Settings</h2>
      <button class="back-btn" @click="onBack">✕</button>
    </header>

    <div class="settings-body">
      <div class="setting-row">
        <label class="setting-label">BGM Volume</label>
        <input
          type="range"
          class="setting-slider"
          min="0"
          max="100"
          :value="settings.bgm_volume"
          @input="onSlider('bgm_volume', $event)"
        />
        <span class="setting-value">{{ settings.bgm_volume }}</span>
      </div>

      <div class="setting-row">
        <label class="setting-label">SFX Volume</label>
        <input
          type="range"
          class="setting-slider"
          min="0"
          max="100"
          :value="settings.sfx_volume"
          @input="onSlider('sfx_volume', $event)"
        />
        <span class="setting-value">{{ settings.sfx_volume }}</span>
      </div>

      <div class="setting-row">
        <label class="setting-label">Text Speed (CPS)</label>
        <input
          type="range"
          class="setting-slider"
          min="10"
          max="200"
          :value="settings.text_speed"
          @input="onSlider('text_speed', $event)"
        />
        <span class="setting-value">{{ settings.text_speed }}</span>
      </div>

      <div class="setting-row">
        <label class="setting-label">Auto Delay (s)</label>
        <input
          type="range"
          class="setting-slider"
          min="0.5"
          max="5.0"
          step="0.1"
          :value="settings.auto_delay"
          @input="onAutoDelay($event)"
        />
        <span class="setting-value">{{ settings.auto_delay.toFixed(1) }}</span>
      </div>

      <div class="setting-row">
        <label class="setting-label">Fullscreen</label>
        <button
          class="toggle-btn"
          :class="{ active: settings.fullscreen }"
          @click="toggleFullscreen"
        >
          {{ settings.fullscreen ? "ON" : "OFF" }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-screen {
  width: 100%;
  height: 100%;
  background: linear-gradient(160deg, #0d0d1a 0%, #1a1a2e 50%, #16213e 100%);
  display: flex;
  flex-direction: column;
  padding: 40px 80px;
  box-sizing: border-box;
}

.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 40px;
}

.settings-title {
  font-family: var(--vn-font-body);
  font-size: 24px;
  font-weight: 400;
  color: #e0e0e0;
  letter-spacing: 3px;
  margin: 0;
}

.back-btn {
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  color: #aaa;
  font-size: 18px;
  width: 40px;
  height: 40px;
  cursor: pointer;
  transition: all 0.2s;
}
.back-btn:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #e0e0e0;
}

.settings-body {
  max-width: 560px;
  margin: 0 auto;
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 28px;
}

.setting-row {
  display: flex;
  align-items: center;
  gap: 16px;
}

.setting-label {
  font-family: var(--vn-font-body);
  font-size: 14px;
  color: #c0c0c0;
  width: 160px;
  flex-shrink: 0;
}

.setting-slider {
  flex: 1;
  -webkit-appearance: none;
  appearance: none;
  height: 4px;
  background: rgba(255, 255, 255, 0.12);
  border-radius: 2px;
  outline: none;
}

.setting-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: rgba(100, 140, 255, 0.7);
  cursor: pointer;
  transition: background 0.2s;
}
.setting-slider::-webkit-slider-thumb:hover {
  background: rgba(100, 140, 255, 1);
}

.setting-value {
  font-family: var(--vn-font-body);
  font-size: 13px;
  color: rgba(255, 255, 255, 0.5);
  width: 48px;
  text-align: right;
}

.toggle-btn {
  padding: 6px 20px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.06);
  color: #aaa;
  font-family: var(--vn-font-body);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}
.toggle-btn.active {
  background: rgba(100, 140, 255, 0.25);
  border-color: rgba(100, 140, 255, 0.4);
  color: #e0e0e0;
}
.toggle-btn:hover {
  background: rgba(100, 140, 255, 0.15);
}
</style>
