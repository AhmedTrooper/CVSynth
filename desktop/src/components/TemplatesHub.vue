<script setup lang="ts">
import { useRoute } from 'vue-router';
import { FileText, Mail, MessageSquare } from '@lucide/vue';

const route = useRoute();

interface SubTab {
  id: string;
  path: string;
  label: string;
  badge: string;
  icon: any;
  matches: string[];
}

const subTabs: SubTab[] = [
  {
    id: 'resumes',
    path: '/templates/resumes',
    label: 'Resume Templates',
    badge: 'LaTeX',
    icon: FileText,
    matches: ['/templates/resumes', '/templates/resume', '/template/resume', '/template/resumes']
  },
  {
    id: 'cover-letters',
    path: '/templates/cover-letters',
    label: 'Cover Letter Templates',
    badge: 'LaTeX',
    icon: Mail,
    matches: ['/templates/cover-letters', '/templates/cover-letter', '/template/cover-letter', '/template/cover-letters']
  },
  {
    id: 'hr-messages',
    path: '/templates/hr-messages',
    label: 'HR Messages for Inbox',
    badge: 'Outreach',
    icon: MessageSquare,
    matches: ['/templates/hr-messages', '/templates/hr-message', '/templates/hr', '/template/hr-message', '/template/hr', '/template/inbox']
  }
];

const isActive = (tab: SubTab) => {
  return tab.matches.some(m => route.path.startsWith(m)) || route.path === tab.path;
};
</script>

<template>
  <div class="templates-hub-wrapper">
    <!-- Sub-tab Bar -->
    <header class="hub-header">
      <div class="hub-nav-track">
        <router-link
          v-for="tab in subTabs"
          :key="tab.id"
          :to="tab.path"
          class="hub-nav-item"
          :class="{ 'active': isActive(tab) }"
        >
          <component :is="tab.icon" :size="16" class="tab-icon" />
          <span class="tab-title">{{ tab.label }}</span>
          <span class="tab-tag">{{ tab.badge }}</span>
        </router-link>
      </div>
    </header>

    <!-- Child View Slot -->
    <main class="hub-content">
      <router-view v-slot="{ Component }">
        <component :is="Component" />
      </router-view>
    </main>
  </div>
</template>

<style scoped>
.templates-hub-wrapper {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  background: var(--bg);
  overflow: hidden;
}

.hub-header {
  height: 52px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  padding: 0 24px 6px 24px;
  background: var(--bg-accent);
  border-bottom: 1px solid var(--line);
  user-select: none;
  overflow-x: auto;
  overflow-y: hidden;
  -webkit-overflow-scrolling: touch;
  scrollbar-width: thin;
}

/* Horizontal scrollbar: 2 unit default, expands to 5 unit on hover/focus */
.hub-header::-webkit-scrollbar {
  height: 2px;
  transition: height 0.15s ease;
}

.hub-header:hover::-webkit-scrollbar,
.hub-header:focus-within::-webkit-scrollbar {
  height: 5px;
}

.hub-header::-webkit-scrollbar-track {
  background: transparent;
  margin: 0 16px;
}

.hub-header::-webkit-scrollbar-thumb {
  background: var(--line);
  border-radius: 4px;
}

.hub-header::-webkit-scrollbar-thumb:hover {
  background: var(--muted);
}

.hub-nav-track {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: max-content;
  margin-bottom: 2px;
}

.hub-nav-item {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 6px 14px;
  min-height: 38px;
  box-sizing: border-box;
  border-radius: 8px;
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--muted);
  text-decoration: none;
  background: transparent;
  border: 1px solid transparent;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.hub-nav-item:hover {
  color: var(--ink);
  background: var(--surface-soft);
}

.hub-nav-item.active {
  color: var(--accent);
  background: var(--surface);
  border-color: var(--line);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}

.tab-icon {
  flex-shrink: 0;
  opacity: 0.8;
}

.hub-nav-item.active .tab-icon {
  opacity: 1;
  color: var(--accent);
}

.tab-title {
  white-space: nowrap;
}

.tab-tag {
  font-size: 0.65rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  padding: 1px 6px;
  border-radius: 4px;
  background: var(--surface-soft);
  color: var(--muted);
  border: 1px solid var(--line);
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.hub-nav-item.active .tab-tag {
  background: var(--accent-soft);
  color: var(--accent);
  border-color: transparent;
}

.hub-content {
  flex: 1;
  height: calc(100% - 48px);
  min-height: 0;
  overflow: hidden;
  position: relative;
}

/* Responsive Breakpoints */
@media (max-width: 959px) {
  .hub-header {
    padding: 0 16px;
  }
}

@media (max-width: 600px) {
  .hub-header {
    padding: 0 10px;
  }

  .hub-nav-track {
    gap: 6px;
  }

  .hub-nav-item {
    padding: 6px 10px;
    font-size: 0.78rem;
    gap: 6px;
    min-height: 38px;
  }

  .tab-tag {
    font-size: 0.6rem;
    padding: 1px 4px;
  }
}

@media (max-width: 360px) {
  .hub-header {
    padding: 0 6px;
  }

  .hub-nav-track {
    gap: 4px;
  }

  .hub-nav-item {
    padding: 5px 8px;
    font-size: 0.74rem;
    gap: 4px;
  }
}
</style>
