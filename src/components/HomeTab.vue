<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();

// Mock data (we will replace this with Pinia store data later)
const savedJobs = ref([
  { id: 'nano_1', title: 'Senior Rust Developer', company: 'TechCorp', date: '2026-05-13' },
  { id: 'nano_2', title: 'Frontend Engineer', company: 'VueMastery', date: '2026-05-12' },
]);

const navigateToJob = (id: string) => {
  router.push(`/job/${id}`);
};
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
        <button class="btn btn-outline">Manage Templates</button>
      </div>
    </div>

    <div class="bento-grid">
      <div class="bento-item large">
        <h3>Recent Applications</h3>
        <div class="job-list-minimal">
          <div class="job-item">
            <span class="job-dot"></span>
            <div class="job-info">
              <span class="j-title">Staff Rust Engineer</span>
              <span class="j-company">OpenSystems</span>
            </div>
            <span class="j-date">2m ago</span>
          </div>
        </div>
      </div>
      <div class="bento-item small">
        <h3>Model Config</h3>
        <p>Active: GPT-4o</p>
        <span class="stat">98% Accuracy</span>
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

/* Bento Grid */
.bento-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 16px;
}

.bento-item {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 16px;
  padding: 20px;
  box-shadow: var(--shadow);
}

.bento-item h3 {
  font-size: 0.85rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--muted);
  margin: 0 0 16px 0;
}

.job-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 0;
  border-bottom: 1px solid var(--line);
}

.j-title { font-weight: 700; display: block; color: var(--ink); }
.j-company { font-size: 0.85rem; color: var(--muted); }
.j-date { margin-left: auto; font-size: 0.8rem; color: #8c857a; }

@media (min-width: 960px) {
  .home-container { padding: 80px 32px 100px; }
  .hero-section { text-align: center; }
  .description { margin: 0 auto 40px; }
  .actions { justify-content: center; }
  .main-title { font-size: 3.1rem; }
  .bento-grid { grid-template-columns: 2fr 1fr; }
}
</style>