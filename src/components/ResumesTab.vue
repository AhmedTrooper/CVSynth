<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useResumesStore } from '../store/resumes';
import { Plus } from '@lucide/vue';

const router = useRouter();
const resumesStore = useResumesStore();

const showNewResumeForm = ref(false);
const newResumeName = ref('');
const newResumeCategory = ref('');
const isCreating = ref(false);

onMounted(async () => {
  await resumesStore.loadAllResumes();
});

const navigateToResume = (resumeId: string) => {
  router.push(`/resume/${resumeId}`);
};

const toggleNewForm = () => {
  showNewResumeForm.value = !showNewResumeForm.value;
  if (!showNewResumeForm.value) {
    newResumeName.value = '';
    newResumeCategory.value = '';
  }
};

const handleCreateResume = async () => {
  if (!newResumeName.value.trim() || !newResumeCategory.value.trim()) {
    return;
  }
  
  isCreating.value = true;
  try {
    // Create an empty template; content will be added in the detail page
    const resumeId = await resumesStore.createNewResume(
      newResumeName.value,
      newResumeCategory.value,
      ''
    );
    showNewResumeForm.value = false;
    newResumeName.value = '';
    newResumeCategory.value = '';
    
    // Navigate to the new resume's detail page
    router.push(`/resume/${resumeId}`);
  } catch (err: any) {
    console.error(err);
  } finally {
    isCreating.value = false;
  }
};
</script>

<template>
  <div class="resumes-container">
    <div class="header">
      <h2>Resume Templates</h2>
      <button class="btn-add" @click="toggleNewForm">
        <Plus :size="18" /> New Template
      </button>
    </div>

    <div v-if="resumesStore.error" class="error-banner">
      {{ resumesStore.error }}
    </div>

    <div v-if="showNewResumeForm" class="form-card">
      <h3>Create New Resume Template</h3>
      <div class="form-group">
        <label>Template Name:</label>
        <input 
          v-model="newResumeName" 
          type="text" 
          placeholder="e.g., Senior Engineer Base"
          class="form-input"
        />
      </div>
      <div class="form-group">
        <label>Category:</label>
        <input 
          v-model="newResumeCategory" 
          type="text" 
          placeholder="e.g., Software Engineering"
          class="form-input"
        />
      </div>
      <div class="form-actions">
        <button 
          class="btn-cancel" 
          @click="toggleNewForm"
        >
          Cancel
        </button>
        <button 
          class="btn-save" 
          @click="handleCreateResume"
          :disabled="isCreating || !newResumeName || !newResumeCategory"
        >
          {{ isCreating ? 'Creating...' : 'Create & Edit' }}
        </button>
      </div>
    </div>

    <div v-if="resumesStore.isLoading" class="loading">
      Loading resumes...
    </div>

    <div v-else-if="resumesStore.resumes.length === 0" class="empty-state">
      <p>No resume templates yet. Create your first one!</p>
    </div>

    <div v-else class="resumes-grid">
      <div 
        v-for="resume in resumesStore.resumes" 
        :key="resume.id"
        class="resume-card"
        @click="navigateToResume(resume.id)"
      >
        <div class="card-header">
          <h3>{{ resume.name }}</h3>
          <span class="category">{{ resume.category }}</span>
        </div>
        <p class="date">{{ new Date(resume.created_at).toLocaleDateString() }}</p>
        <p class="id">ID: {{ resume.id }}</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.resumes-container {
  padding: 24px 20px 40px;
  max-width: 1200px;
  margin: 0 auto;
}

.header {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 24px;
}

.header h2 {
  margin: 0;
  font-size: 1.7rem;
  color: var(--ink);
}

.btn-add {
  background-color: var(--accent);
  color: #fff;
  border: none;
  padding: 12px 18px;
  border-radius: 12px;
  font-weight: 700;
  cursor: pointer;
  transition: 0.2s;
  width: fit-content;
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn-add:hover { background-color: #0a6b5e; }

.error-banner {
  background: rgba(180, 35, 24, 0.1);
  border: 1px solid rgba(180, 35, 24, 0.2);
  border-radius: 10px;
  padding: 12px 16px;
  margin-bottom: 16px;
  color: var(--warning);
}

.form-card {
  background-color: var(--surface);
  border: 1px solid var(--line);
  border-radius: 16px;
  padding: 20px;
  margin-bottom: 24px;
  box-shadow: var(--shadow);
}

.form-card h3 {
  margin-top: 0;
  color: var(--ink);
}

.form-group {
  margin-bottom: 14px;
  display: flex;
  flex-direction: column;
}

.form-group label {
  color: var(--accent);
  font-weight: 700;
  margin-bottom: 6px;
  font-size: 0.78rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.form-input {
  background-color: var(--surface);
  border: 1px solid var(--line);
  border-radius: 10px;
  padding: 12px 14px;
  color: var(--ink);
  font-size: 1rem;
  outline: none;
  transition: 0.2s;
}

.form-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px rgba(11, 123, 107, 0.2);
}

.form-actions {
  display: flex;
  gap: 10px;
  justify-content: flex-end;
  margin-top: 16px;
}

.btn-cancel, .btn-save {
  padding: 10px 18px;
  border-radius: 10px;
  font-weight: 700;
  border: none;
  cursor: pointer;
  transition: 0.2s;
}

.btn-cancel {
  background: var(--surface-soft);
  color: var(--muted);
}

.btn-cancel:hover { color: var(--ink); }

.btn-save {
  background-color: var(--accent);
  color: #fff;
}

.btn-save:hover:not(:disabled) { background-color: #0a6b5e; }
.btn-save:disabled { opacity: 0.5; cursor: not-allowed; }

.loading {
  text-align: center;
  color: var(--muted);
  padding: 30px;
}

.empty-state {
  text-align: center;
  color: var(--muted);
  padding: 40px 20px;
  font-size: 1rem;
}

.resumes-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 16px;
}

.resume-card {
  background-color: var(--surface);
  border: 1px solid var(--line);
  border-radius: 16px;
  padding: 20px;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: var(--shadow);
}

.resume-card:hover {
  border-color: var(--accent);
  transform: translateY(-2px);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: start;
  margin-bottom: 10px;
}

.card-header h3 {
  margin: 0;
  color: var(--ink);
  font-size: 1.1rem;
}

.category {
  background: rgba(11, 123, 107, 0.12);
  color: var(--accent);
  padding: 4px 10px;
  border-radius: 999px;
  font-size: 0.72rem;
  font-weight: 700;
}

.date {
  color: var(--muted);
  font-size: 0.9rem;
  margin: 6px 0 0 0;
}

.id {
  color: #8c857a;
  font-size: 0.8rem;
  margin: 4px 0 0 0;
  font-family: 'Monaco', 'Menlo', monospace;
}

@media (min-width: 960px) {
  .resumes-container { padding: 40px 32px 60px; }
  .header { flex-direction: row; align-items: center; justify-content: space-between; }
  .resumes-grid { grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); }
}
</style>
