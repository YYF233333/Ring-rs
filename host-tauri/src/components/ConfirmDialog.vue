<script setup lang="ts">
import { computed } from "vue";
import { useTheme } from "../composables/useTheme";

defineProps<{
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
}>();

const emit = defineEmits<{
  confirm: [];
  cancel: [];
}>();

const { asset } = useTheme();
const overlayUrl = computed(() => asset("confirm_overlay"));
const frameUrl = computed(() => asset("frame"));
</script>

<template>
  <Teleport to="body">
    <div class="confirm-overlay" @click.self="emit('cancel')">
      <img
        v-if="overlayUrl"
        class="confirm-overlay-bg"
        :src="overlayUrl"
        alt=""
      />

      <div class="confirm-card">
        <img
          v-if="frameUrl"
          class="confirm-frame-bg"
          :src="frameUrl"
          alt=""
        />
        <div class="confirm-inner">
          <h3 class="confirm-title">{{ title }}</h3>
          <p class="confirm-message">{{ message }}</p>
          <div class="confirm-actions">
            <button class="btn btn-cancel" @click="emit('cancel')">
              {{ cancelText ?? "取消" }}
            </button>
            <button class="btn btn-confirm" @click="emit('confirm')">
              {{ confirmText ?? "确认" }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.confirm-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9000;
  backdrop-filter: blur(4px);
}

.confirm-overlay-bg {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  pointer-events: none;
}

.confirm-card {
  position: relative;
  min-width: clamp(300px, 30vw, 440px);
  max-width: 500px;
}

.confirm-frame-bg {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: fill;
  pointer-events: none;
}

/* Fallback when no frame image */
.confirm-card:not(:has(.confirm-frame-bg)) {
  background: #1e1e3a;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
}

.confirm-inner {
  position: relative;
  z-index: 1;
  padding: clamp(24px, 3vw, 40px);
}

.confirm-title {
  margin: 0 0 12px;
  font-family: var(--vn-font-body);
  font-size: clamp(15px, 1.2vw, 20px);
  color: var(--vn-color-ui-text, #e0e0e0);
}

.confirm-message {
  margin: 0 0 24px;
  font-family: var(--vn-font-body);
  font-size: clamp(12px, 0.9vw, 15px);
  color: #aaa;
  line-height: 1.6;
}

.confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.btn {
  padding: 8px 22px;
  border: none;
  border-radius: 6px;
  font-family: var(--vn-font-body);
  font-size: clamp(12px, 0.85vw, 14px);
  cursor: pointer;
  transition: background 0.2s;
}

.btn-cancel {
  background: rgba(255, 255, 255, 0.08);
  color: #aaa;
}
.btn-cancel:hover {
  background: rgba(255, 255, 255, 0.15);
}

.btn-confirm {
  background: rgba(255, 153, 0, 0.25);
  color: var(--vn-color-ui-text, #e0e0e0);
}
.btn-confirm:hover {
  background: rgba(255, 153, 0, 0.45);
}
</style>
