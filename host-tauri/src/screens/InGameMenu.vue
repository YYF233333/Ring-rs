<script setup lang="ts">
const emit = defineEmits<{
  resume: [];
  save: [];
  load: [];
  history: [];
  settings: [];
  title: [];
}>();
</script>

<template>
  <div class="ingame-menu-overlay" @click.self="emit('resume')">
    <nav class="ingame-menu">
      <button class="igm-btn" @click="emit('resume')">Resume</button>
      <button class="igm-btn" @click="emit('save')">Save</button>
      <button class="igm-btn" @click="emit('load')">Load</button>
      <button class="igm-btn" @click="emit('history')">History</button>
      <button class="igm-btn" @click="emit('settings')">Settings</button>
      <button class="igm-btn igm-btn-danger" @click="emit('title')">
        Return to Title
      </button>
    </nav>
  </div>
</template>

<style scoped>
.ingame-menu-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 500;
  backdrop-filter: blur(6px);
}

.ingame-menu {
  display: flex;
  flex-direction: column;
  gap: 12px;
  align-items: center;
}

.igm-btn {
  width: 220px;
  padding: 12px 0;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  color: #c0c0c0;
  font-family: var(--vn-font-body);
  font-size: 15px;
  letter-spacing: 2px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.igm-btn:hover {
  background: rgba(100, 140, 255, 0.15);
  border-color: rgba(100, 140, 255, 0.3);
  color: #e0e0e0;
}

.igm-btn-danger:hover {
  background: rgba(200, 60, 60, 0.2);
  border-color: rgba(200, 60, 60, 0.4);
}
</style>
