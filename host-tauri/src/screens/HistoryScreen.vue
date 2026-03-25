<script setup lang="ts">
import { ref, onMounted } from "vue";
import { callBackend } from "../composables/useBackend";

export interface HistoryEntry {
  speaker: string | null;
  text: string;
}

const emit = defineEmits<{
  back: [];
}>();

const entries = ref<HistoryEntry[]>([]);

onMounted(async () => {
  try {
    entries.value = await callBackend<HistoryEntry[]>("get_history");
  } catch {
    entries.value = [];
  }
});
</script>

<template>
  <div class="history-screen">
    <header class="history-header">
      <h2 class="history-title">History</h2>
      <button class="back-btn" @click="emit('back')">✕</button>
    </header>

    <div class="history-list">
      <div
        v-for="(entry, i) in entries"
        :key="i"
        class="history-entry"
      >
        <span v-if="entry.speaker" class="entry-speaker">{{ entry.speaker }}</span>
        <span class="entry-text">{{ entry.text }}</span>
      </div>
      <div v-if="entries.length === 0" class="history-empty">
        暂无历史记录
      </div>
    </div>
  </div>
</template>

<style scoped>
.history-screen {
  width: 100%;
  height: 100%;
  background: linear-gradient(160deg, #0d0d1a 0%, #1a1a2e 50%, #16213e 100%);
  display: flex;
  flex-direction: column;
  padding: 40px 60px;
  box-sizing: border-box;
}

.history-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 28px;
  flex-shrink: 0;
}

.history-title {
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

.history-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding-right: 8px;
}

.history-list::-webkit-scrollbar {
  width: 4px;
}
.history-list::-webkit-scrollbar-track {
  background: transparent;
}
.history-list::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
}

.history-entry {
  padding: 12px 16px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 8px;
  border-left: 3px solid rgba(100, 140, 255, 0.2);
}

.entry-speaker {
  font-family: var(--vn-font-body);
  font-size: 13px;
  color: rgba(100, 160, 255, 0.8);
  margin-right: 12px;
  font-weight: 500;
}

.entry-text {
  font-family: var(--vn-font-body);
  font-size: 14px;
  color: #c0c0c0;
  line-height: 1.6;
}

.history-empty {
  text-align: center;
  padding: 60px 0;
  font-family: var(--vn-font-body);
  font-size: 14px;
  color: rgba(255, 255, 255, 0.2);
}
</style>
