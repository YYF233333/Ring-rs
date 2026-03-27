<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { useEngine } from "../composables/useEngine";
import type { SaveInfo } from "../types/render-state";

const props = defineProps<{
  mode: "save" | "load";
  saves: SaveInfo[];
}>();

const emit = defineEmits<{
  save: [slot: number];
  load: [slot: number];
}>();

const { deleteSave, getThumbnail } = useEngine();

const SLOTS_PER_PAGE = 6;
const currentPage = ref(0);

const totalPages = computed(() => Math.max(1, Math.ceil((SLOTS_PER_PAGE * 5) / SLOTS_PER_PAGE)));

const thumbnails: Record<number, string | null> = reactive({});

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

async function loadThumbnails() {
  for (const { slot, info } of visibleSlots.value) {
    if (info && !(slot in thumbnails)) {
      getThumbnail(slot).then((b64) => {
        thumbnails[slot] = b64;
      });
    }
  }
}

watch([currentPage, () => props.saves], loadThumbnails, { immediate: true });

function thumbnailSrc(slot: number): string | undefined {
  const b64 = thumbnails[slot];
  return b64 ? `data:image/png;base64,${b64}` : undefined;
}

function onSlotClick(slot: number) {
  if (props.mode === "save") {
    emit("save", slot);
  } else {
    emit("load", slot);
  }
}

async function onDelete(slot: number, ev: Event) {
  ev.stopPropagation();
  delete thumbnails[slot];
  await deleteSave(slot);
}

function formatTimestamp(ts: string): string {
  const secs = Number.parseInt(ts, 10);
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
  <div class="save-load-content">
    <h2 class="sl-title">{{ mode === "save" ? "保存" : "读取" }}</h2>

    <div class="slots-grid">
      <div
        v-for="item in visibleSlots"
        :key="item.slot"
        class="slot-card"
        :class="{ empty: !item.info }"
        @click="onSlotClick(item.slot)"
      >
        <div class="slot-thumb">
          <img
            v-if="item.info && thumbnailSrc(item.slot)"
            :src="thumbnailSrc(item.slot)"
            class="slot-thumb-img"
            draggable="false"
          />
          <span v-else-if="!item.info" class="slot-empty-label">空</span>
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
          <div class="slot-chapter">槽位 {{ item.slot }}</div>
        </div>
        <button
          v-if="item.info"
          class="slot-delete"
          title="删除存档"
          @click="onDelete(item.slot, $event)"
        >
          ×
        </button>
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
.save-load-content {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.sl-title {
  font-family: var(--vn-font-body);
  font-size: clamp(16px, 1.4vw, 24px);
  font-weight: 400;
  color: var(--vn-color-ui-text, #e0e0e0);
  letter-spacing: 3px;
  margin: 0 0 2vh 0;
}

.slots-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: clamp(8px, 1vw, 16px);
  flex: 1;
}

.slot-card {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  flex-direction: column;
  position: relative;
}

.slot-card:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: var(--vn-color-hover, rgba(255, 153, 0, 0.3));
}

.slot-card.empty {
  opacity: 0.5;
}
.slot-card.empty:hover {
  opacity: 0.8;
}

.slot-thumb {
  height: clamp(60px, 10vh, 120px);
  background: rgba(0, 0, 0, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.slot-thumb-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.slot-empty-label {
  font-family: var(--vn-font-body);
  font-size: 13px;
  color: rgba(255, 255, 255, 0.2);
}

.slot-number {
  font-family: var(--vn-font-body);
  font-size: 13px;
  color: rgba(255, 255, 255, 0.3);
}

.slot-meta {
  padding: 8px 10px;
}

.slot-chapter {
  font-family: var(--vn-font-body);
  font-size: clamp(11px, 0.8vw, 14px);
  color: #c0c0c0;
  margin-bottom: 3px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.slot-time {
  font-family: var(--vn-font-body);
  font-size: clamp(10px, 0.7vw, 12px);
  color: rgba(255, 255, 255, 0.35);
}

.slot-delete {
  position: absolute;
  top: 4px;
  right: 4px;
  width: 22px;
  height: 22px;
  background: rgba(200, 60, 60, 0.3);
  border: none;
  border-radius: 4px;
  color: rgba(255, 255, 255, 0.6);
  font-size: 14px;
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.slot-card:hover .slot-delete {
  opacity: 1;
}

.slot-delete:hover {
  background: rgba(200, 60, 60, 0.6);
  color: #fff;
}

.pagination {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 16px;
  margin-top: 1.5vh;
}

.page-btn {
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  color: #aaa;
  font-size: 18px;
  width: 32px;
  height: 32px;
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
  font-size: 12px;
  color: rgba(255, 255, 255, 0.4);
}
</style>
