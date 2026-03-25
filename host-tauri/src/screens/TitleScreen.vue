<script setup lang="ts">
const emit = defineEmits<{
  "new-game": [];
  continue: [];
  load: [];
  settings: [];
  quit: [];
}>();
</script>

<template>
  <div class="title-screen">
    <div class="title-area">
      <h1 class="game-title">Ring Engine</h1>
    </div>

    <nav class="menu-list">
      <button class="menu-btn" @click="emit('new-game')">New Game</button>
      <button class="menu-btn" @click="emit('continue')">Continue</button>
      <button class="menu-btn" @click="emit('load')">Load</button>
      <button class="menu-btn" @click="emit('settings')">Settings</button>
      <button class="menu-btn" @click="emit('quit')">Quit</button>
    </nav>

    <div class="version-tag">v0.1.0</div>
  </div>
</template>

<style scoped>
.title-screen {
  width: 100%;
  height: 100%;
  background: linear-gradient(160deg, #0d0d1a 0%, #1a1a2e 50%, #16213e 100%);
  display: flex;
  flex-direction: column;
  align-items: center;
  position: relative;
}

.title-area {
  margin-top: 22vh;
  text-align: center;
}

.game-title {
  font-family: var(--vn-font-display);
  font-size: 56px;
  font-weight: 300;
  color: #e0e0e0;
  letter-spacing: 12px;
  margin: 0;
  text-shadow: 0 0 40px rgba(120, 140, 255, 0.25);
}

.menu-list {
  margin-top: 8vh;
  display: flex;
  flex-direction: column;
  gap: 12px;
  align-items: center;
}

.menu-btn {
  width: 240px;
  padding: 12px 0;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  color: #c0c0c0;
  font-family: var(--vn-font-body);
  font-size: 16px;
  letter-spacing: 2px;
  cursor: pointer;
  transition: all 0.25s ease;
}

.menu-btn:hover {
  background: rgba(100, 140, 255, 0.15);
  border-color: rgba(100, 140, 255, 0.3);
  color: #e0e0e0;
  transform: translateX(4px);
}

.menu-btn:active {
  transform: translateX(2px);
  background: rgba(100, 140, 255, 0.25);
}

.version-tag {
  position: absolute;
  bottom: 20px;
  right: 28px;
  font-family: var(--vn-font-body);
  font-size: 12px;
  color: rgba(255, 255, 255, 0.2);
}
</style>
