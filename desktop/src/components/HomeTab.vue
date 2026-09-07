<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useSettingsStore } from '../store/settings';
import { useJobsStore, Job } from '../store/jobs';
import { Motion, AnimatePresence } from 'motion-v';

import { Activity, Plus, FileText, LayoutGrid, Mail, MessageSquare, Send } from '@lucide/vue';

const router = useRouter();
const route = useRoute();
const settingsStore = useSettingsStore();
const jobsStore = useJobsStore();
const settingsError = ref('');
const isLoadingSettings = ref(false);

// Tooltip State
const activeTooltip = ref<string | null>(null);

const savedJobs = ref<Job[]>([]);

const navigateToJob = (id: string) => {
  router.push(`/job/${id}`);
};

const refreshData = async () => {
  isLoadingSettings.value = true;
  settingsError.value = '';
  try {
    await settingsStore.loadSettings();
    await settingsStore.loadProviderKeyStatus(settingsStore.selectedAiProvider);
    savedJobs.value = await jobsStore.loadAllJobs();
  } catch (err: any) {
    settingsError.value = err?.message || 'Failed to load data.';
  } finally {
    isLoadingSettings.value = false;
  }
};

onMounted(refreshData);

watch(() => route.fullPath, async () => {
  if (route.name === 'Home') await refreshData();
});
</script>

<template>
  <div class="home-container">
    <div class="hero">
      <div class="status-indicator">
        <Activity :size="12" class="status-icon" />
        <span class="text">Engine Ready</span>
      </div>
      
      <h1 class="main-title">Craft your professional narrative.</h1>
      
      <p class="subtitle">
        Surgical AI tailoring for high-performance LaTeX resumes.
      </p>

      <div class="actions">
        <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'new-app'" @mouseleave="activeTooltip = null">
          <button class="btn-primary" @click="$router.push('/parse')" title="New Application" aria-label="New Application"><Plus :size="18" class="action-icon" /></button>
          <AnimatePresence>
            <Motion
              v-if="activeTooltip === 'new-app'"
              :initial="{ opacity: 0, y: 5, scale: 0.9 }"
              :animate="{ opacity: 1, y: 0, scale: 1 }"
              :exit="{ opacity: 0, y: 5, scale: 0.9 }"
              :transition="{ duration: 0.15 }"
              class="floating-message tooltip-top"
            >
              New Application
            </Motion>
          </AnimatePresence>
        </div>
        <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'resumes'" @mouseleave="activeTooltip = null">
          <button class="btn-secondary" @click="$router.push('/templates/resumes')" title="Resume Templates" aria-label="Resume Templates"><FileText :size="18" class="action-icon" /></button>
          <AnimatePresence>
            <Motion
              v-if="activeTooltip === 'resumes'"
              :initial="{ opacity: 0, y: 5, scale: 0.9 }"
              :animate="{ opacity: 1, y: 0, scale: 1 }"
              :exit="{ opacity: 0, y: 5, scale: 0.9 }"
              :transition="{ duration: 0.15 }"
              class="floating-message tooltip-top"
            >
              Resume Templates
            </Motion>
          </AnimatePresence>
        </div>
        <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'cls'" @mouseleave="activeTooltip = null">
          <button class="btn-secondary" @click="$router.push('/templates/cover-letters')" title="Cover Letter Templates" aria-label="Cover Letter Templates"><Mail :size="18" class="action-icon" /></button>
          <AnimatePresence>
            <Motion
              v-if="activeTooltip === 'cls'"
              :initial="{ opacity: 0, y: 5, scale: 0.9 }"
              :animate="{ opacity: 1, y: 0, scale: 1 }"
              :exit="{ opacity: 0, y: 5, scale: 0.9 }"
              :transition="{ duration: 0.15 }"
              class="floating-message tooltip-top"
            >
              CL Templates
            </Motion>
          </AnimatePresence>
        </div>
        <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'hr-messages'" @mouseleave="activeTooltip = null">
          <button class="btn-secondary" @click="$router.push('/templates/hr-messages')" title="HR Templates" aria-label="HR Templates"><MessageSquare :size="18" class="action-icon" /></button>
          <AnimatePresence>
            <Motion
              v-if="activeTooltip === 'hr-messages'"
              :initial="{ opacity: 0, y: 5, scale: 0.9 }"
              :animate="{ opacity: 1, y: 0, scale: 1 }"
              :exit="{ opacity: 0, y: 5, scale: 0.9 }"
              :transition="{ duration: 0.15 }"
              class="floating-message tooltip-top"
            >
              HR Templates
            </Motion>
          </AnimatePresence>
        </div>
        <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'outreach'" @mouseleave="activeTooltip = null">
          <button class="btn-secondary" @click="$router.push('/outreach')" title="Direct Outreach" aria-label="Direct Outreach"><Send :size="18" class="action-icon" /></button>
          <AnimatePresence>
            <Motion
              v-if="activeTooltip === 'outreach'"
              :initial="{ opacity: 0, y: 5, scale: 0.9 }"
              :animate="{ opacity: 1, y: 0, scale: 1 }"
              :exit="{ opacity: 0, y: 5, scale: 0.9 }"
              :transition="{ duration: 0.15 }"
              class="floating-message tooltip-top"
            >
              Direct Outreach
            </Motion>
          </AnimatePresence>
        </div>
      </div>
    </div>

    <div class="recent-section">
      <div class="section-header">
        <h3>RECENT APPLICATIONS</h3>
        <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'all-jobs'" @mouseleave="activeTooltip = null">
          <button class="link-btn" @click="$router.push('/jobs')" title="All Applications" aria-label="All Applications"><LayoutGrid :size="18" class="action-icon" /></button>
          <AnimatePresence>
            <Motion
              v-if="activeTooltip === 'all-jobs'"
              :initial="{ opacity: 0, y: 5, scale: 0.9 }"
              :animate="{ opacity: 1, y: 0, scale: 1 }"
              :exit="{ opacity: 0, y: 5, scale: 0.9 }"
              :transition="{ duration: 0.15 }"
              class="floating-message tooltip-bottom-left"
            >
              All Applications
            </Motion>
          </AnimatePresence>
        </div>
      </div>

      <div v-if="savedJobs.length === 0" class="empty-state">
        No active applications. Start by parsing a job description.
      </div>
      
      <div v-else class="list">
        <button
          v-for="job in savedJobs.slice(0, 8)"
          :key="job.id"
          class="item"
          @click="navigateToJob(job.id)"
        >
          <div class="item-main">
            <span class="item-title">{{ job.job_title }}</span>
            <span class="item-meta">{{ job.company_name }}</span>
          </div>
          <span class="item-date">{{ (job.created_at?.split(' ')[0] || '').split('T')[0] }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.home-container {
  padding: 40px;
  max-width: 800px;
  margin: 0 auto;
  box-sizing: border-box;
  width: 100%;
}

.hero {
  margin-bottom: 40px;
}

.status-indicator {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
  font-size: 0.65rem;
  font-weight: 700;
  text-transform: uppercase;
  color: var(--muted);
}
.status-icon { color: var(--accent); flex-shrink: 0; }

.main-title {
  font-size: 2rem;
  font-weight: 700;
  color: var(--ink);
  margin: 0 0 8px 0;
  letter-spacing: -0.01em;
  word-break: break-word;
}

.subtitle {
  font-size: 0.9rem;
  color: var(--muted);
  margin-bottom: 24px;
  word-break: break-word;
}

.actions {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.btn-tooltip-wrapper {
  position: relative;
  display: flex;
  flex-shrink: 0;
}

.btn-primary, .btn-secondary {
  padding: 10px;
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-lg);
  font-size: 0.75rem;
  font-weight: 600;
  cursor: pointer;
  transition: 0.15s;
  flex-shrink: 0;
}

.action-icon {
  flex-shrink: 0;
}

.btn-primary { background: var(--accent); color: #fff; border: none; }
.btn-primary:hover { opacity: 0.9; }

.btn-secondary { background: var(--surface-soft); color: var(--ink); border: 1px solid var(--line); }
.btn-secondary:hover { border-color: var(--muted); }

.recent-section {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: var(--radius-lg);
  overflow: hidden;
}

.section-header {
  padding: 12px 16px;
  background: var(--bg-accent);
  border-bottom: 1px solid var(--line);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.section-header h3 {
  font-size: 0.65rem;
  color: var(--muted);
  margin: 0;
  letter-spacing: 0.05em;
}

.link-btn {
  background: none;
  border: none;
  color: var(--accent);
  font-weight: 700;
  font-size: 0.65rem;
  cursor: pointer;
  padding: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.list { display: flex; flex-direction: column; }

.item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 16px;
  background: none;
  border: none;
  border-bottom: 1px solid var(--line);
  width: 100%;
  text-align: left;
  cursor: pointer;
  transition: 0.15s;
}
.item:last-child { border-bottom: none; }
.item:hover { background: var(--surface-soft); }

.item-main {
  min-width: 0;
  flex: 1;
  padding-right: 12px;
}
.item-title { display: block; font-size: 0.85rem; font-weight: 600; color: var(--ink); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.item-meta { font-size: 0.75rem; color: var(--muted); display: block; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.item-date { font-size: 0.7rem; color: var(--muted); font-family: monospace; white-space: nowrap; flex-shrink: 0; }

.empty-state {
  padding: 32px;
  text-align: center;
  color: var(--muted);
  font-size: 0.8rem;
}

/* =======================================================================
   Tablet Styles (601px - 959px)
   ======================================================================= */
@media (max-width: 959px) and (min-width: 601px) {
  .home-container {
    padding: 28px 24px;
  }
  .hero {
    margin-bottom: 28px;
  }
  .main-title {
    font-size: 1.65rem;
  }
  .subtitle {
    font-size: 0.88rem;
  }
  .actions {
    gap: 12px;
  }
}

/* =======================================================================
   Mobile Styles (<= 600px):
   Production-grade touch targets (min 44px), full-size readable icons,
   and left-to-right horizontal scrolling where content overflows.
   ======================================================================= */
@media (max-width: 600px) {
  .home-container {
    padding: 16px 14px;
  }
  
  .hero {
    margin-bottom: 20px;
  }

  .status-indicator {
    font-size: 0.65rem;
    gap: 6px;
    margin-bottom: 8px;
  }

  .main-title {
    font-size: clamp(1.2rem, 5.5vw, 1.45rem);
    line-height: 1.25;
    margin-bottom: 6px;
  }

  .subtitle {
    font-size: 0.82rem;
    line-height: 1.4;
    margin-bottom: 16px;
  }

  /* Left-to-right scrollable action tray with visible sleek scrollbar */
  .actions {
    gap: 10px;
    flex-wrap: nowrap;
    overflow-x: auto;
    overflow-y: hidden;
    -webkit-overflow-scrolling: touch;
    scroll-behavior: smooth;
    padding-bottom: 6px;
    scrollbar-width: thin;
    scrollbar-color: var(--line) transparent;
  }

  .actions::-webkit-scrollbar {
    height: 3px;
    display: block;
  }

  .actions::-webkit-scrollbar-track {
    background: transparent;
  }

  .actions::-webkit-scrollbar-thumb {
    background: var(--line);
    border-radius: 3px;
  }

  .actions::-webkit-scrollbar-thumb:hover {
    background: var(--accent);
  }

  /* Maintain production-grade touch targets (44px) and full-sized icons */
  .btn-primary, .btn-secondary {
    width: 44px;
    height: 44px;
    min-width: 44px;
    border-radius: var(--radius-lg);
  }

  .btn-primary:active, .btn-secondary:active {
    transform: scale(0.92);
  }

  .action-icon {
    width: 19px;
    height: 19px;
  }

  .floating-message {
    display: none !important;
  }

  /* Recent Applications Card */
  .recent-section {
    border-radius: var(--radius-lg);
  }

  .section-header {
    padding: 10px 14px;
  }

  .section-header h3 {
    font-size: 0.65rem;
  }

  .link-btn {
    min-width: 36px;
    min-height: 36px;
    padding: 6px;
  }

  .link-btn:active {
    transform: scale(0.92);
  }

  .item {
    min-height: 48px;
    padding: 10px 14px;
  }

  .item:active {
    background: var(--surface-soft);
    transform: scale(0.99);
  }

  .item-title {
    font-size: 0.85rem;
  }

  .item-meta {
    font-size: 0.75rem;
  }

  .item-date {
    font-size: 0.68rem;
    padding: 2px 6px;
    border-radius: 4px;
    background: var(--surface-soft);
    border: 1px solid var(--line);
  }

  .empty-state {
    padding: 24px 14px;
    font-size: 0.78rem;
  }
}

/* =======================================================================
   Ultra-compact Mobile (<= 340px width):
   Keep touch targets and icons legible, refine container margins
   ======================================================================= */
@media (max-width: 340px) {
  .home-container {
    padding: 12px 10px;
  }

  .actions {
    gap: 8px;
  }

  .btn-primary, .btn-secondary {
    width: 42px;
    height: 42px;
    min-width: 42px;
  }

  .action-icon {
    width: 18px;
    height: 18px;
  }

  .section-header {
    padding: 8px 10px;
  }

  .item {
    padding: 8px 10px;
  }
}

/* =======================================================================
   Compact Viewport Height (<= 450px height):
   Compress vertical whitespace while preserving readable typography
   ======================================================================= */
@media (max-height: 450px) {
  .home-container {
    padding-top: 8px;
    padding-bottom: 8px;
  }

  .hero {
    margin-bottom: 12px;
  }

  .status-indicator {
    margin-bottom: 4px;
  }

  .main-title {
    margin-bottom: 4px;
  }

  .subtitle {
    margin-bottom: 10px;
  }
}
</style>
