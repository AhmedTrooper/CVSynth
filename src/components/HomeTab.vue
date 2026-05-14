<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useSettingsStore } from '../store/settings';
import { useJobsStore, Job } from '../store/jobs';

const router = useRouter();
const route = useRoute();
const settingsStore = useSettingsStore();
const jobsStore = useJobsStore();
const settingsError = ref('');
const isLoadingSettings = ref(false);

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

onMounted(async () => {
  await refreshData();
});

watch(
  () => route.fullPath,
  async () => {
    if (route.name === 'Home') {
      await refreshData();
    }
  }
);
</script>

<template>
  <div class="home-container">
    <div class="hero-section">
      <div class="status-pill">
        <span class="pulse"></span> 
        Engine Status: Ready
      </div>
      
      <h1 class="main-title">
        Crafting resumes with <br />
        <span class="serif-italic">Intentional Design.</span>
      </h1>
      
      <p class="description">
        CVSynth uses refined AI models to distill job descriptions into precise 
        data points, helping you build a targeted professional narrative.
      </p>

      <div class="actions">
        <button class="btn btn-dark" @click="$router.push('/parse')">New Application</button>
        <button class="btn btn-outline" @click="$router.push('/resumes')">Manage Templates</button>
      </div>
    </div>

    <div class="recent-jobs-section">
      <div class="section-header">
        <h3>Recent Applications</h3>
        <button class="link-btn" @click="$router.push('/jobs')">View All &rarr;</button>
      </div>

      <div v-if="savedJobs.length === 0" class="empty-block">
        No jobs yet. Paste a job description to start.
      </div>
      
      <div v-else class="job-list-minimal">
        <button
          v-for="job in savedJobs.slice(0, 5)"
          :key="job.id"
          class="job-item"
          @click="navigateToJob(job.id)"
          type="button"
        >
          <span class="job-dot"></span>
          <div class="job-info">
            <span class="j-title">{{ job.job_title }}</span>
            <span class="j-company">{{ job.company_name }}</span>
          </div>
          <span class="j-date">{{ job.created_at?.split(' ')[0] }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.home-container {
  max-width: 1000px;
  margin: 0 auto;
  padding: 32px 20px 60px;
}

.hero-section {
  text-align: left;
  margin-bottom: 48px;
}

.status-pill {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  background: var(--surface-soft);
  padding: 6px 12px;
  border-radius: 999px;
  font-size: 0.78rem;
  font-weight: 700;
  color: var(--muted);
  margin-bottom: 18px;
  border: 1px solid var(--line);
}

.pulse {
  width: 6px;
  height: 6px;
  background: #10b981;
  border-radius: 50%;
  animation: pulse-ring 2s infinite;
}

@keyframes pulse-ring {
  0% { transform: scale(0.9); opacity: 0.7; }
  50% { transform: scale(1.1); opacity: 1; }
  100% { transform: scale(0.9); opacity: 0.7; }
}

.main-title {
  font-size: 2.4rem;
  font-weight: 800;
  color: var(--ink);
  letter-spacing: -0.03em;
  line-height: 1.1;
  margin-bottom: 18px;
}

.serif-italic {
  font-family: 'Merriweather', serif;
  font-style: italic;
  font-weight: 400;
  color: var(--muted);
}

.description {
  font-size: 1rem;
  color: var(--muted);
  max-width: 540px;
  margin: 0 0 28px;
  line-height: 1.6;
}

.actions { display: flex; gap: 12px; flex-wrap: wrap; }

.btn {
  padding: 12px 18px;
  border-radius: 12px;
  font-size: 0.95rem;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-dark {
  background: var(--accent);
  color: #fff;
  border: none;
}

.btn-dark:hover { background: #0a6b5e; transform: translateY(-1px); }

.btn-outline {
  background: var(--surface);
  color: var(--ink);
  border: 1px solid var(--line);
}

.btn-outline:hover { background: var(--surface-soft); }

.recent-jobs-section {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 20px;
  padding: 24px;
  box-shadow: var(--shadow);
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.section-header h3 {
  font-size: 0.9rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--muted);
  margin: 0;
}

.job-list-minimal {
  display: flex;
  flex-direction: column;
}

.job-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 14px 0;
  border-bottom: 1px solid var(--line);
  background: transparent;
  border-top: none;
  border-left: none;
  border-right: none;
  width: 100%;
  text-align: left;
  cursor: pointer;
  transition: 0.2s;
}

.job-item:last-child {
  border-bottom: none;
}

.job-dot {
  width: 8px;
  height: 8px;
  background: var(--accent);
  border-radius: 50%;
}

.j-title { font-weight: 700; display: block; color: var(--ink); font-size: 1.05rem; }
.j-company { font-size: 0.9rem; color: var(--muted); }
.j-date { margin-left: auto; font-size: 0.85rem; color: #8c857a; font-family: monospace; }

.job-item:hover {
  transform: translateX(4px);
}

.empty-block {
  color: var(--muted);
  font-size: 0.95rem;
  padding: 20px 0;
  text-align: center;
}

.link-btn {
  background: none;
  border: none;
  color: var(--accent);
  font-weight: 700;
  font-size: 0.85rem;
  cursor: pointer;
}

@media (min-width: 960px) {
  .home-container { padding: 80px 32px 100px; }
  .hero-section { text-align: center; }
  .main-title { font-size: 3.4rem; }
}
</style>
