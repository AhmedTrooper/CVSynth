<script setup lang="ts">
import { Motion } from 'motion-v';

const tabs = [
  { path: '/', label: 'Home', icon: '🏠' },
  { path: '/jobs', label: 'Jobs', icon: '💼' },
  { path: '/resumes', label: 'Templates', icon: '📄' },
  { path: '/settings', label: 'Settings', icon: '⚙️' },
];
</script>

<template>
  <div class="app-container">
    <aside class="sidebar">
      <div class="logo-section">
        <div class="logo-dot"></div>
      </div>
      
      <nav class="nav-menu">
        <router-link 
          v-for="tab in tabs" 
          :key="tab.path"
          :to="tab.path"
          class="nav-item"
          active-class="active"
        >
          <div class="icon-wrapper">
            <span class="icon">{{ tab.icon }}</span>
          </div>
          <span class="nav-label">{{ tab.label }}</span>
        </router-link>
      </nav>
    </aside>

    <main class="content-area">
      <router-view v-slot="{ Component, route }">
        <transition mode="out-in">
          <Motion
            :key="route.path"
            :initial="{ opacity: 0, y: 5 }"
            :animate="{ opacity: 1, y: 0 }"
            :transition="{ duration: 0.15, ease: 'easeOut' }"
            class="route-wrapper"
          >
            <component :is="Component" />
          </Motion>
        </transition>
      </router-view>
    </main>
  </div>
</template>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100%;
  background: var(--bg);
  color: var(--ink);
  overflow: hidden;
}

.sidebar {
  order: 2;
  background: var(--bg-accent);
  border-top: 1px solid var(--line);
  z-index: 100;
  display: flex;
}

.logo-section {
  display: none;
}

.nav-menu {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  width: 100%;
  padding: 4px;
}

.nav-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 8px 0;
  color: var(--muted);
  text-decoration: none;
  font-size: 0.65rem;
  font-weight: 500;
  transition: 0.15s;
}

.nav-item:hover {
  color: var(--ink);
}

.nav-item.active {
  color: var(--accent);
}

.icon-wrapper {
  font-size: 1.2rem;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.content-area {
  flex: 1;
  overflow: hidden;
  position: relative;
}

.route-wrapper {
  height: 100%;
  width: 100%;
  overflow-y: auto;
}

@media (min-width: 960px) {
  .app-container {
    flex-direction: row;
  }

  .sidebar {
    order: 0;
    width: 48px;
    height: 100vh;
    flex-direction: column;
    border-top: none;
    border-right: 1px solid var(--line);
    padding: 12px 0;
  }

  .logo-section {
    display: flex;
    justify-content: center;
    margin-bottom: 24px;
  }

  .logo-dot {
    width: 6px;
    height: 6px;
    background: var(--accent);
    border-radius: 50%;
  }

  .nav-menu {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 0;
  }

  .nav-item {
    width: 100%;
    padding: 8px 0;
    position: relative;
  }

  .nav-label {
    display: none;
  }

  .nav-item.active::before {
    content: '';
    position: absolute;
    left: 0;
    top: 8px;
    bottom: 8px;
    width: 2px;
    background: var(--accent);
  }

  .icon-wrapper {
    font-size: 1.1rem;
  }
}
</style>