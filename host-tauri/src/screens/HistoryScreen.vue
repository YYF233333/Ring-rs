<script setup lang="ts">
import type { HistoryEntry } from "../types/render-state";

defineProps<{
  entries: HistoryEntry[];
}>();
</script>

<template>
  <div class="history-content">
    <h2 class="history-title">历史</h2>

    <div class="history-list">
      <div
        v-for="(entry, i) in entries"
        :key="i"
        class="history-entry"
      >
        <div class="entry-name-col">
          <span v-if="entry.speaker" class="entry-speaker">{{ entry.speaker }}</span>
        </div>
        <div class="entry-text-col">
          <span class="entry-text">{{ entry.text }}</span>
        </div>
      </div>
      <div v-if="entries.length === 0" class="history-empty">
        暂无历史记录
      </div>
    </div>
  </div>
</template>

<style scoped>
.history-content {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.history-title {
  font-family: var(--vn-font-body);
  font-size: clamp(16px, 1.4vw, 24px);
  font-weight: 400;
  color: var(--vn-color-ui-text, #e0e0e0);
  letter-spacing: 3px;
  margin: 0 0 2vh 0;
  flex-shrink: 0;
}

.history-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 6px;
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
  display: flex;
  gap: 12px;
  padding: 8px 12px;
  background: rgba(255, 255, 255, 0.02);
  border-radius: 6px;
  border-left: 2px solid rgba(255, 255, 255, 0.06);
}

.entry-name-col {
  width: clamp(60px, 8vw, 140px);
  flex-shrink: 0;
  text-align: right;
}

.entry-speaker {
  font-family: var(--vn-font-body);
  font-size: clamp(11px, 0.8vw, 14px);
  color: var(--vn-color-hover, rgba(255, 153, 0, 0.8));
  font-weight: 500;
}

.entry-text-col {
  flex: 1;
}

.entry-text {
  font-family: var(--vn-font-body);
  font-size: clamp(12px, 0.9vw, 15px);
  color: #c0c0c0;
  line-height: 1.6;
}

.history-empty {
  text-align: center;
  padding: 40px 0;
  font-family: var(--vn-font-body);
  font-size: 13px;
  color: rgba(255, 255, 255, 0.2);
}
</style>
