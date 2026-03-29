<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { useSettings } from "../composables/useSettings";

const { settings, loadSettings, saveSettings, updateSetting } = useSettings();

onMounted(() => {
  loadSettings();
});

onUnmounted(() => {
  saveSettings();
});

function onSlider(key: "bgm_volume" | "sfx_volume" | "text_speed", e: Event) {
  const val = Number.parseFloat((e.target as HTMLInputElement).value);
  updateSetting(key, val);
}

function onAutoDelay(e: Event) {
  const val = Number.parseFloat((e.target as HTMLInputElement).value);
  updateSetting("auto_delay", val);
}

function toggleFullscreen() {
  updateSetting("fullscreen", !settings.value.fullscreen);
  void saveSettings();
}
</script>

<template>
  <div class="settings-content">
    <h2 class="settings-title">设置</h2>

    <div class="settings-body">
      <div class="setting-row">
        <label class="setting-label">BGM 音量</label>
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
        <label class="setting-label">效果音量</label>
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
        <label class="setting-label">文字速度 (CPS)</label>
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
        <label class="setting-label">自动延迟 (秒)</label>
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
        <label class="setting-label">全屏</label>
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
.settings-content {
  width: 100%;
  height: 100%;
}

.settings-title {
  font-family: var(--vn-font-body);
  font-size: clamp(16px, 1.4vw, 24px);
  font-weight: 400;
  color: var(--vn-color-ui-text, #e0e0e0);
  letter-spacing: 3px;
  margin: 0 0 3vh 0;
}

.settings-body {
  max-width: 480px;
  display: flex;
  flex-direction: column;
  gap: clamp(16px, 2.5vh, 32px);
}

.setting-row {
  display: flex;
  align-items: center;
  gap: 16px;
}

.setting-label {
  font-family: var(--vn-font-body);
  font-size: clamp(12px, 0.9vw, 15px);
  color: #c0c0c0;
  width: clamp(100px, 10vw, 180px);
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
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--vn-color-hover, rgba(255, 153, 0, 0.8));
  cursor: pointer;
  transition: background 0.2s;
}
.setting-slider::-webkit-slider-thumb:hover {
  background: var(--vn-color-hover, #ff9900);
}

.setting-value {
  font-family: var(--vn-font-body);
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
  width: 40px;
  text-align: right;
}

.toggle-btn {
  padding: 6px 18px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.06);
  color: #aaa;
  font-family: var(--vn-font-body);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}
.toggle-btn.active {
  background: rgba(255, 153, 0, 0.2);
  border-color: rgba(255, 153, 0, 0.4);
  color: #e0e0e0;
}
.toggle-btn:hover {
  background: rgba(255, 153, 0, 0.15);
}
</style>
