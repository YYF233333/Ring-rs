<script setup lang="ts">
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
</script>

<template>
  <Teleport to="body">
    <div class="confirm-overlay" @click.self="emit('cancel')">
      <div class="confirm-card">
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

.confirm-card {
  background: #1e1e3a;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 32px;
  min-width: 340px;
  max-width: 440px;
}

.confirm-title {
  margin: 0 0 12px;
  font-family: var(--vn-font-body);
  font-size: 18px;
  color: #e0e0e0;
}

.confirm-message {
  margin: 0 0 24px;
  font-family: var(--vn-font-body);
  font-size: 14px;
  color: #aaa;
  line-height: 1.6;
}

.confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.btn {
  padding: 8px 24px;
  border: none;
  border-radius: 8px;
  font-family: var(--vn-font-body);
  font-size: 14px;
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
  background: rgba(100, 140, 255, 0.3);
  color: #e0e0e0;
}
.btn-confirm:hover {
  background: rgba(100, 140, 255, 0.5);
}
</style>
