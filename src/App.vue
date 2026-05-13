<script setup lang="ts">
import { useRoute } from 'vue-router';

const route = useRoute();

const tabs = [
  { path: '/', label: 'Saved Jobs', icon: '🏠' },
  { path: '/settings', label: 'Settings', icon: '⚙️' },
];
</script>

<template>
  <div class="app-container">
    <aside class="sidebar">
      <div class="logo-container">
        <div class="logo-icon"></div>
      </div>
      
      <nav class="nav-menu">
        <router-link 
          v-for="tab in tabs" 
          :key="tab.path"
          :to="tab.path"
          class="nav-button"
          :class="{ active: route.path === tab.path }"
        >
          <span class="icon">{{ tab.icon }}</span> 
          <span class="tab-label">{{ tab.label }}</span>
        </router-link>
      </nav>
    </aside>

    <main class="content-area">
      <router-view />
    </main>
  </div>
</template>

<style scoped>
/* PREMIUM DARK GRID VIBE */
.app-container {
  display: flex;
  height: 100vh;
  width: 100vw;
  background-color: #030303; /* Pitch black */
  /* The subtle grid pattern */
  background-image: 
    linear-gradient(rgba(255, 255, 255, 0.03) 1px, transparent 1px),
    linear-gradient(90deg, rgba(255, 255, 255, 0.03) 1px, transparent 1px);
  background-size: 40px 40px;
  color: #ededed;
  font-family: 'Inter', system-ui, -apple-system, sans-serif;
  overflow: hidden;
}

/* --- SLIM, FLAT SIDEBAR --- */
.sidebar {
  width: 80px;
  background: #000000;
  border-right: 1px solid rgba(255, 255, 255, 0.08);
  display: flex;
  flex-direction: column;
  align-items: center;
  z-index: 10;
}

.logo-container {
  padding: 25px 0;
  display: flex;
  justify-content: center;
  width: 100%;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.logo-icon {
  width: 28px;
  height: 28px;
  background: #00e599; /* Neon Emerald Accent */
  border-radius: 6px;
  box-shadow: 0 0 15px rgba(0, 229, 153, 0.4);
}

/* --- NAVIGATION --- */
.nav-menu {
  display: flex;
  flex-direction: column;
  padding: 20px 0;
  gap: 15px;
  width: 100%;
  align-items: center;
}

.nav-button {
  position: relative;
  background: transparent;
  border: 1px solid transparent;
  color: #888888;
  padding: 0;
  width: 48px;
  height: 48px;
  display: flex;
  justify-content: center;
  align-items: center;
  border-radius: 10px;
  cursor: pointer;
  text-decoration: none;
  transition: all 0.2s ease;
}

.nav-button:hover {
  background: rgba(255, 255, 255, 0.05);
  color: #ededed;
}

.nav-button.active {
  background: rgba(0, 229, 153, 0.1);
  border: 1px solid rgba(0, 229, 153, 0.3);
  color: #00e599;
}

.nav-button .icon {
  font-size: 1.4rem;
  filter: grayscale(100%) brightness(120%);
  transition: all 0.2s ease;
}

.nav-button.active .icon {
  filter: none;
}

/* --- TOOLTIP --- */
.tab-label {
  position: absolute;
  left: 100%;
  margin-left: 15px;
  background: #111111;
  border: 1px solid rgba(255, 255, 255, 0.1);
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 0.85rem;
  font-weight: 500;
  color: #ededed;
  white-space: nowrap;
  opacity: 0;
  pointer-events: none;
  transform: translateX(-5px);
  transition: all 0.2s ease;
  z-index: 100;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
}

.nav-button:hover .tab-label {
  opacity: 1;
  transform: translateX(0);
}

/* --- MAIN CONTENT AREA --- */
.content-area {
  flex-grow: 1;
  padding: 0; 
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

::-webkit-scrollbar { width: 8px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: #333; border-radius: 4px; }
::-webkit-scrollbar-thumb:hover { background: #555; }
</style>