<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useJobsStore, Job } from '../store/jobs';

const router = useRouter();
const jobsStore = useJobsStore();

const allJobs = ref<Job[]>([]);
const searchQuery = ref('');
const statusFilter = ref('All');
const sortBy = ref('date-desc');

const statuses = ['All', 'Drafting', 'Applied', 'Interviewing', 'Offer', 'Rejected'];

const loadJobs = async () => {
  allJobs.value = await jobsStore.loadAllJobs();
};

onMounted(loadJobs);

const filteredAndSortedJobs = computed(() => {
  let result = [...allJobs.value];

  // Search Filter
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase();
    result = result.filter(j => 
      j.job_title.toLowerCase().includes(q) || 
      j.company_name.toLowerCase().includes(q)
    );
  }

  // Status Filter
  if (statusFilter.value !== 'All') {
    result = result.filter(j => j.status === statusFilter.value);
  }

  // Sort
  result.sort((a, b) => {
    switch (sortBy.value) {
      case 'date-desc':
        return new Date(b.created_at || 0).getTime() - new Date(a.created_at || 0).getTime();
      case 'date-asc':
        return new Date(a.created_at || 0).getTime() - new Date(b.created_at || 0).getTime();
      case 'title':
        return a.job_title.localeCompare(b.job_title);
      case 'company':
        return a.company_name.localeCompare(b.company_name);
      default:
        return 0;
    }
  });

  return result;
});

const navigateToJob = (id: string) => {
  router.push(`/job/${id}`);
};

const getStatusClass = (status: string) => {
  return `status-badge ${status.toLowerCase()}`;
};
</script>

<template>
  <div class="jobs-container">
    <header class="page-header">
      <div class="title-group">
        <h1>Application Vault</h1>
        <p class="subtitle">Track and manage your professional opportunities.</p>
      </div>
      <button class="btn-primary" @click="$router.push('/parse')">
        + New Application
      </button>
    </header>

    <div class="filters-bar">
      <div class="search-box">
        <span class="icon">🔍</span>
        <input v-model="searchQuery" placeholder="Search by title or company..." />
      </div>

      <div class="controls">
        <div class="filter-group">
          <label>Status</label>
          <select v-model="statusFilter">
            <option v-for="s in statuses" :key="s" :value="s">{{ s }}</option>
          </select>
        </div>

        <div class="filter-group">
          <label>Sort By</label>
          <select v-model="sortBy">
            <option value="date-desc">Newest First</option>
            <option value="date-asc">Oldest First</option>
            <option value="title">Job Title</option>
            <option value="company">Company</option>
          </select>
        </div>
      </div>
    </div>

    <div class="jobs-grid">
      <div v-if="jobsStore.isLoading" class="loading-state">
        Scanning vault...
      </div>
      <div v-else-if="filteredAndSortedJobs.length === 0" class="empty-state">
        <div class="empty-icon">📂</div>
        <h3>No applications found</h3>
        <p>Try adjusting your search or filters.</p>
      </div>
      
      <div 
        v-for="job in filteredAndSortedJobs" 
        :key="job.id"
        class="job-card"
        @click="navigateToJob(job.id)"
      >
        <div class="card-top">
          <span :class="getStatusClass(job.status)">{{ job.status }}</span>
          <span class="date">{{ job.created_at?.split(' ')[0] }}</span>
        </div>
        
        <h2 class="job-title">{{ job.job_title }}</h2>
        <p class="company-name">{{ job.company_name }}</p>
        
        <div class="tags">
          <span class="tag">{{ job.work_model }}</span>
          <span class="tag">{{ job.employment_type }}</span>
        </div>
        
        <div class="card-footer">
          <span class="view-link">View Details &rarr;</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.jobs-container {
  padding: 40px;
  max-width: 1200px;
  margin: 0 auto;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 32px;
}

.page-header h1 { font-size: 2.2rem; margin: 0; color: var(--ink); }
.subtitle { color: var(--muted); margin: 8px 0 0; }

.btn-primary {
  background: var(--accent);
  color: white;
  border: none;
  padding: 12px 24px;
  border-radius: 12px;
  font-weight: 700;
  cursor: pointer;
  transition: 0.2s;
}

.btn-primary:hover { transform: translateY(-1px); box-shadow: 0 4px 12px rgba(11, 123, 107, 0.2); }

.filters-bar {
  background: var(--surface);
  border: 1px solid var(--line);
  padding: 20px;
  border-radius: 16px;
  display: flex;
  gap: 24px;
  margin-bottom: 32px;
  align-items: center;
  box-shadow: var(--shadow);
}

.search-box {
  flex: 1;
  position: relative;
  display: flex;
  align-items: center;
  background: var(--surface-soft);
  border-radius: 10px;
  padding: 0 16px;
  border: 1px solid var(--line);
}

.search-box input {
  width: 100%;
  padding: 12px 8px;
  background: none;
  border: none;
  color: var(--ink);
  outline: none;
  font-size: 0.95rem;
}

.controls { display: flex; gap: 20px; }

.filter-group { display: flex; flex-direction: column; gap: 6px; }
.filter-group label {
  font-size: 0.7rem;
  font-weight: 800;
  text-transform: uppercase;
  color: var(--accent);
  letter-spacing: 0.05em;
}

.filter-group select {
  background: var(--surface-soft);
  border: 1px solid var(--line);
  padding: 8px 12px;
  border-radius: 8px;
  color: var(--ink);
  font-weight: 600;
  cursor: pointer;
}

.jobs-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(min(100%, 300px), 1fr));
  gap: 24px;
}

.job-card {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 16px;
  padding: 24px;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow);
}

.job-card:hover {
  transform: translateY(-4px);
  border-color: var(--accent);
  box-shadow: 0 8px 24px rgba(0,0,0,0.06);
}

.card-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.status-badge {
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 0.75rem;
  font-weight: 700;
  text-transform: uppercase;
}

.status-badge.drafting { background: #f3f4f6; color: #4b5563; }
.status-badge.applied { background: #e0f2fe; color: #0369a1; }
.status-badge.interviewing { background: #fef3c7; color: #92400e; }
.status-badge.offer { background: #dcfce7; color: #166534; }
.status-badge.rejected { background: #fee2e2; color: #991b1b; }

.date { font-size: 0.8rem; color: var(--muted); font-family: monospace; }

.job-title { font-size: 1.25rem; margin: 0; color: var(--ink); font-weight: 800; }
.company-name { color: var(--accent); font-weight: 700; margin: 4px 0 16px; }

.tags { display: flex; gap: 8px; margin-bottom: auto; }
.tag {
  background: var(--surface-soft);
  color: var(--muted);
  font-size: 0.75rem;
  padding: 4px 8px;
  border-radius: 6px;
  font-weight: 600;
}

.card-footer {
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid var(--line);
}

.view-link {
  font-size: 0.85rem;
  font-weight: 700;
  color: var(--muted);
  transition: 0.2s;
}

.job-card:hover .view-link { color: var(--accent); }

.loading-state, .empty-state {
  grid-column: 1 / -1;
  text-align: center;
  padding: 80px 0;
  color: var(--muted);
}

.empty-icon { font-size: 3rem; margin-bottom: 16px; opacity: 0.5; }

@media (max-width: 768px) {
  .jobs-container {
    padding: 20px;
  }
  .page-header { flex-direction: column; gap: 20px; }
  .filters-bar { flex-direction: column; align-items: stretch; }
  .controls { flex-direction: column; }
}

@media (max-width: 480px) {
  .jobs-container {
    padding: 16px;
  }
}
</style>
