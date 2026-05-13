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
  padding: 100px 40px;
}

.hero-section {
  text-align: center;
  margin-bottom: 80px;
}

.status-pill {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  background: #f0f0ed;
  padding: 6px 14px;
  border-radius: 100px;
  font-size: 0.8rem;
  font-weight: 600;
  color: #555550;
  margin-bottom: 24px;
  border: 1px solid #e5e5e0;
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
  font-size: 3.5rem;
  font-weight: 700;
  color: #1a1a1a;
  letter-spacing: -0.04em;
  line-height: 1.1;
  margin-bottom: 24px;
}

.serif-italic {
  font-family: 'Playfair Display', serif; /* Or any premium serif */
  font-style: italic;
  font-weight: 400;
  color: #71716e;
}

.description {
  font-size: 1.1rem;
  color: #71716e;
  max-width: 540px;
  margin: 0 auto 40px;
  line-height: 1.6;
}

.actions { display: flex; gap: 12px; justify-content: center; }

.btn {
  padding: 12px 24px;
  border-radius: 8px;
  font-size: 0.95rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-dark {
  background: #1a1a1a;
  color: white;
  border: none;
}

.btn-dark:hover { background: #333; transform: translateY(-1px); }

.btn-outline {
  background: white;
  color: #1a1a1a;
  border: 1px solid #e5e5e0;
}

.btn-outline:hover { background: #f9f9f7; }

/* Bento Grid */
.bento-grid {
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: 20px;
}

.bento-item {
  background: white;
  border: 1px solid #e5e5e0;
  border-radius: 16px;
  padding: 24px;
}

.bento-item h3 {
  font-size: 0.9rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: #a1a19a;
  margin: 0 0 20px 0;
}

.job-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 0;
  border-bottom: 1px solid #f4f4f2;
}

.j-title { font-weight: 600; display: block; }
.j-company { font-size: 0.85rem; color: #71716e; }
.j-date { margin-left: auto; font-size: 0.8rem; color: #a1a19a; }
</style>