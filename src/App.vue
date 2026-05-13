<script setup lang="ts">
const tabs = [
  { path: '/', label: 'Saved Jobs', icon: '🏠' },
  { path: '/resumes', label: 'Resume Templates', icon: '📄' },
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
          active-class="active"
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
.app-container {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  width: 100vw;
  background: var(--bg);
  color: var(--ink);
}

.sidebar {
  order: 2;
  background: var(--surface);
  border-top: 1px solid var(--line);
  position: sticky;
  bottom: 0;
  z-index: 10;
}

.logo-container {
  display: none;
}

.logo-icon {
  width: 36px;
  height: 36px;
  background: var(--ink);
  border-radius: 10px;
}

.nav-menu {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 6px;
  padding: 10px 12px 14px;
}

.nav-button {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 10px 6px;
  border-radius: 14px;
  color: var(--muted);
  text-decoration: none;
  font-size: 0.75rem;
  font-weight: 600;
  letter-spacing: 0.02em;
  transition: background 0.2s ease, color 0.2s ease, transform 0.2s ease;
}

.nav-button:hover {
  background: var(--surface-soft);
  color: var(--ink);
}

.nav-button.active {
  background: var(--bg-accent);
  color: var(--ink);
  box-shadow: var(--shadow);
  transform: translateY(-1px);
}

.icon {
  font-size: 1.2rem;
}

.tab-label {
  position: static;
  background: transparent;
  color: inherit;
  padding: 0;
  border-radius: 0;
  opacity: 1;
  pointer-events: auto;
  font-size: 0.7rem;
}

.content-area {
  flex: 1;
  overflow-y: auto;
}

@media (min-width: 960px) {
  .app-container {
    flex-direction: row;
  }

  .sidebar {
    order: 0;
    position: static;
    width: 88px;
    min-height: 100vh;
    border-top: none;
    border-right: 1px solid var(--line);
  }

  .logo-container {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 26px 0 10px;
  }

  .nav-menu {
    display: flex;
    flex-direction: column;
    padding: 18px 12px;
    gap: 12px;
  }

  .nav-button {
    padding: 12px 6px;
  }

  .tab-label {
    font-size: 0.65rem;
  }
}
</style>