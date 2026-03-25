<script setup lang="ts">
import { ref } from "vue";

interface ToastItem {
  id: number;
  message: string;
  type: "success" | "error" | "info";
}

const items = ref<ToastItem[]>([]);
let nextId = 0;

function show(message: string, type: ToastItem["type"] = "info") {
  const id = nextId++;
  items.value.push({ id, message, type });
  setTimeout(() => {
    items.value = items.value.filter((t) => t.id !== id);
  }, 3000);
}

defineExpose({ show });
</script>

<template>
  <Teleport to="body">
    <div class="toast-container">
      <TransitionGroup name="toast">
        <div
          v-for="item in items"
          :key="item.id"
          class="toast-item"
          :class="'toast-' + item.type"
        >
          {{ item.message }}
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<style scoped>
.toast-container {
  position: fixed;
  top: clamp(16px, 6vh, 72px);
  right: 24px;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 8px;
  pointer-events: none;
}

.toast-item {
  padding: 8px 20px;
  border-radius: 6px;
  font-family: var(--vn-font-body);
  font-size: clamp(11px, 0.8vw, 14px);
  color: #e0e0e0;
  backdrop-filter: blur(12px);
  pointer-events: auto;
}

.toast-success {
  background: rgba(46, 125, 50, 0.85);
}
.toast-error {
  background: rgba(198, 40, 40, 0.85);
}
.toast-info {
  background: rgba(30, 30, 60, 0.85);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}
.toast-enter-from {
  opacity: 0;
  transform: translateX(16px);
}
.toast-leave-to {
  opacity: 0;
  transform: translateX(8px);
}
</style>
