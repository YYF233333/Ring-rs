<script setup lang="ts">
import { computed, ref } from "vue";
import type { SaveInfo } from "../types/render-state";

const props = defineProps<{
  mode: "save" | "load";
  saves: SaveInfo[];
}>();

const emit = defineEmits<{
  back: [];
  save: [slot: number];
  load: [slot: number];
}>();

const SLOTS_PER_PAGE = 9;
const currentPage = ref(0);

const totalPages = computed(() => Math.max(1, Math.ceil((SLOTS_PER_PAGE * 3) / SLOTS_PER_PAGE)));

const visibleSlots = computed(() => {
  const start = currentPage.value * SLOTS_PER_PAGE + 1;
  const slots: { slot: number; info: SaveInfo | null }[] = [];
  for (let i = 0; i < SLOTS_PER_PAGE; i++) {
    const slotNum = start + i;
    const info = props.saves.find((s) => s.slot === slotNum) ?? null;
    slots.push({ slot: slotNum, info });
  }
  return slots;
});

function onSlotClick(slot: number) {
  if (props.mode === "save") {
    emit("save", slot);
  } else {
    emit("load", slot);
  }
}

function formatTimestamp(ts: string): string {
  const secs = parseInt(ts, 10);
  if (Number.isNaN(secs)) return ts;
  const d = new Date(secs * 1000);
  return d.toLocaleString("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}
</script>

<template>
  <div class="save-load-screen">
    <header class="sl-header">
      <div class="sl-tabs">
        <span class="sl-tab" :class="{ active: mode === 'save' }">Save</span>
        <span class="sl-tab" :class="{ active: mode === 'load' }">Load</span>
      </div>
      <button class="back-btn" @click="emit('back')">✕</button>
    </header>

    <div class="slots-grid">
      <div
        v-for="item in visibleSlots"
        :key="item.slot"
        class="slot-card"
        :class="{ empty: !item.info }"
        @click="onSlotClick(item.slot)"
      >
        <div class="slot-thumb">
          <span v-if="!item.info" class="slot-empty-label">Empty</span>
          <span v-else class="slot-number">#{{ item.slot }}</span>
        </div>
        <div v-if="item.info" class="slot-meta">
          <div class="slot-chapter">
            {{ item.info.chapter_title ?? "---" }}
          </div>
          <div class="slot-time">
            {{ formatTimestamp(item.info.timestamp) }}
          </div>
        </div>
        <div v-else class="slot-meta">
          <div class="slot-chapter">Slot {{ item.slot }}</div>
        </div>
      </div>
    </div>

    <div class="pagination">
      <button
        class="page-btn"
        :disabled="currentPage === 0"
        @click="currentPage--"
      >
        ‹
      </button>
      <span class="page-info">{{ currentPage + 1 }} / {{ totalPages }}</span>
      <button
        class="page-btn"
        :disabled="currentPage >= totalPages - 1"
        @click="currentPage++"
      >
        ›
      </button>
    </div>
  </div>
</template>

<style scoped>
.save-load-screen {
  width: 100%;
  height: 100%;
  background: linear-gradient(160deg, #0d0d1a 0%, #1a1a2e 50%, #16213e 100%);
  display: flex;
  flex-direction: column;
  padding: 40px 60px;
  box-sizing: border-box;
}

.sl-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 32px;
}

.sl-tabs {
  display: flex;
  gap: 24px;
}

.sl-tab {
  font-family: var(--vn-font-body);
  font-size: 20px;
  color: rgba(255, 255, 255, 0.3);
  letter-spacing: 2px;
  padding-bottom: 4px;
}

.sl-tab.active {
  color: #e0e0e0;
  border-bottom: 2px solid rgba(100, 140, 255, 0.6);
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

.slots-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  flex: 1;
}

.slot-card {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 10px;
  overflow: hidden;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  flex-direction: column;
}

.slot-card:hover {
  background: rgba(100, 140, 255, 0.08);
  border-color: rgba(100, 140, 255, 0.25);
}

.slot-card.empty {
  opacity: 0.5;
}
.slot-card.empty:hover {
  opacity: 0.8;
}

.slot-thumb {
  height: 100px;
  background: rgba(0, 0, 0, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
}

.slot-empty-label {
  font-family: var(--vn-font-body);
  font-size: 14px;
  color: rgba(255, 255, 255, 0.2);
}

.slot-number {
  font-family: var(--vn-font-body);
  font-size: 14px;
  color: rgba(255, 255, 255, 0.3);
}

.slot-meta {
  padding: 10px 12px;
}

.slot-chapter {
  font-family: var(--vn-font-body);
  font-size: 13px;
  color: #c0c0c0;
  margin-bottom: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.slot-time {
  font-family: var(--vn-font-body);
  font-size: 11px;
  color: rgba(255, 255, 255, 0.35);
}

.pagination {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 16px;
  margin-top: 20px;
}

.page-btn {
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  color: #aaa;
  font-size: 20px;
  width: 36px;
  height: 36px;
  cursor: pointer;
  transition: all 0.2s;
}
.page-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.12);
  color: #e0e0e0;
}
.page-btn:disabled {
  opacity: 0.3;
  cursor: default;
}

.page-info {
  font-family: var(--vn-font-body);
  font-size: 13px;
  color: rgba(255, 255, 255, 0.4);
}
</style>
