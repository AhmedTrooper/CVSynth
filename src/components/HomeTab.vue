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
      <div class="badge">
        <span class="dot"></span>
        CVSynth Engine 1.0
      </div>

      <h1 class="headline">
        Tailor your resume with<br/>
        <span class="highlight">absolute precision.</span>
      </h1>

      <p class="subheadline">
        Paste a job description, select your base LaTeX template, and let the AI extract requirements to generate an honest, targeted resume instantly.
      </p>

      <div class="button-group">
        <button class="btn btn-primary">Start Building &rarr;</button>
        <button class="btn btn-secondary">Explore Templates</button>
      </div>
    </div>

    <div class="pipeline-section">
      <div class="section-header">
        <h2>Recent Opportunities</h2>
        <button class="btn-text">+ New Job</button>
      </div>
      
      <div class="job-grid" v-if="savedJobs.length > 0">
        <div v-for="job in savedJobs" :key="job.id" class="job-card">
          <h3>{{ job.title }}</h3>
          <p class="company">🏢 {{ job.company }}</p>
          <p class="date">📅 {{ job.date }}</p>
          <button class="action-btn" @click="navigateToJob(job.id)">View Details</button>
        </div>
      </div>
      
      <div class="empty-state" v-else>
        <p>No jobs added yet. Click "Start Building" to parse your first JD.</p>
      </div>
    </div>

  </div>
</template>

<style scoped>
.home-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 80px 40px;
  min-height: 100%;
}

/* --- HERO SECTION --- */
.hero-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  max-width: 800px;
  margin-bottom: 80px;
}

.badge {
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(0, 229, 153, 0.05);
  border: 1px solid rgba(0, 229, 153, 0.2);
  color: #00e599;
  padding: 6px 16px;
  border-radius: 100px;
  font-size: 0.85rem;
  font-weight: 500;
  margin-bottom: 30px;
}

.badge .dot {
  width: 6px;
  height: 6px;
  background-color: #00e599;
  border-radius: 50%;
  box-shadow: 0 0 8px #00e599;
}

.headline {
  font-size: 4rem;
  font-weight: 800;
  line-height: 1.1;
  margin: 0 0 20px 0;
  color: #ffffff;
  letter-spacing: -0.03em;
}

.highlight {
  color: #00e599;
  text-shadow: 0 0 40px rgba(0, 229, 153, 0.3); 
}

.subheadline {
  font-size: 1.2rem;
  color: #a1a1aa; 
  max-width: 600px;
  line-height: 1.6;
  margin: 0 0 40px 0;
}

/* Buttons */
.button-group {
  display: flex;
  gap: 16px;
}

.btn {
  padding: 12px 24px;
  border-radius: 8px;
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-primary {
  background-color: #00e599;
  color: #000000;
  border: 1px solid #00e599;
  font-weight: 600;
}

.btn-primary:hover {
  background-color: #00c785;
  border-color: #00c785;
}

.btn-secondary {
  background-color: #111111;
  color: #ededed;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.btn-secondary:hover {
  background-color: #1a1a1a;
  border-color: rgba(255, 255, 255, 0.2);
}

/* --- BOTTOM PIPELINE SECTION --- */
.pipeline-section {
  width: 100%;
  max-width: 1000px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  padding-top: 40px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.section-header h2 {
  font-size: 1.5rem;
  font-weight: 600;
  margin: 0;
}

.btn-text {
  background: transparent;
  color: #00e599;
  border: none;
  font-weight: 500;
  cursor: pointer;
}

.btn-text:hover {
  text-decoration: underline;
}

.job-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 20px;
}

.job-card {
  background-color: #0a0a0a;
  padding: 24px;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  transition: transform 0.2s ease, border-color 0.2s ease;
}

.job-card:hover {
  transform: translateY(-2px);
  border-color: rgba(0, 229, 153, 0.3);
}

.job-card h3 { margin: 0 0 12px 0; color: #ededed; font-size: 1.1rem; }
.company, .date { margin: 6px 0; font-size: 0.9rem; color: #888888; }

.action-btn {
  margin-top: 18px;
  width: 100%;
  padding: 10px;
  background-color: rgba(0, 229, 153, 0.1);
  border: 1px solid rgba(0, 229, 153, 0.2);
  border-radius: 6px;
  color: #00e599;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.action-btn:hover { background-color: rgba(0, 229, 153, 0.2); }

.empty-state {
  background: rgba(255, 255, 255, 0.02);
  border: 1px dashed rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 40px;
  text-align: center;
  color: #71717a;
}
</style>