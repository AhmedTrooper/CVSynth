<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useJobsStore } from '../store/jobs';

const router = useRouter();
const jobsStore = useJobsStore();
const rawJobDescription = ref('');
const jobUrl = ref('');

const handleParse = async () => {
  if (!rawJobDescription.value.trim()) return;
  
  try {
    const slug = await jobsStore.parseNewJob(rawJobDescription.value, jobUrl.value);
    // On success, instantly route them to the dynamic detail view for this job!
    router.push(`/job/${slug}`);
  } catch (e) {
    console.error(e);
  }
};
</script>

<template>
  <div class="parser-container">
    <div class="header">
      <button class="back-btn" @click="router.push('/')">&larr; Back</button>
      <h2>Parse New Job</h2>
    </div>

    <div class="editor-layout">
      <div class="input-section">
        <div class="url-group">
          <label>Job Listing URL (Optional)</label>
          <input 
            v-model="jobUrl" 
            type="url" 
            placeholder="https://linkedin.com/jobs/view/..."
            class="url-input"
          />
        </div>

        <label>Paste Raw Job Description</label>
        <textarea 
          v-model="rawJobDescription" 
          placeholder="Paste the requirements, responsibilities, and company details here..."
          spellcheck="false"
        ></textarea>
      </div>

      <div class="action-section">
        <div class="info-card">
          <h3>What happens next?</h3>
          <p>The AI will extract the core requirements, title, and company name to build a structured JSON profile. This profile will be used to accurately tailor your base resume.</p>
        </div>
        
        <div class="error-box" v-if="jobsStore.error">
          {{ jobsStore.error }}
        </div>

        <button 
          class="btn-primary parse-btn" 
          @click="handleParse" 
          :disabled="jobsStore.isLoading || !rawJobDescription"
        >
          {{ jobsStore.isLoading ? '⚙️ AI is Analyzing...' : 'Extract Data &rarr;' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.parser-container {
  padding: 24px 20px 40px;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
}

.header h2 { margin: 0; font-weight: 700; color: var(--ink); }

.back-btn {
  background: var(--surface);
  border: 1px solid var(--line);
  color: var(--muted);
  padding: 8px 14px;
  border-radius: 10px;
  cursor: pointer;
  transition: 0.2s;
}

.back-btn:hover { color: var(--ink); border-color: var(--accent); }

.editor-layout {
  display: flex;
  flex-direction: column;
  gap: 20px;
  flex-grow: 1;
  min-height: 0;
}

.input-section {
  flex: 2;
  display: flex;
  flex-direction: column;
}

.url-group {
  margin-bottom: 20px;
}

.url-input {
  width: 100%;
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 10px;
  padding: 12px 16px;
  color: var(--ink);
  font-size: 0.95rem;
  transition: 0.2s;
  outline: none;
}

.url-input:focus { border-color: var(--accent); box-shadow: 0 0 0 2px rgba(11, 123, 107, 0.2); }

label {
  color: var(--accent);
  font-weight: 700;
  margin-bottom: 8px;
  font-size: 0.85rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

textarea {
  flex-grow: 1;
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 14px;
  padding: 16px;
  color: var(--ink);
  font-family: 'Monaco', 'Menlo', monospace;
  resize: none;
  outline: none;
  transition: 0.2s;
  min-height: 240px;
}

textarea:focus { border-color: var(--accent); box-shadow: 0 0 0 2px rgba(11, 123, 107, 0.2); }

.action-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.info-card {
  background: var(--surface);
  border: 1px solid var(--line);
  padding: 18px;
  border-radius: 14px;
  box-shadow: var(--shadow);
}

.info-card h3 { color: var(--ink); margin: 0 0 10px 0; font-size: 1.05rem; }
.info-card p { color: var(--muted); font-size: 0.95rem; line-height: 1.5; margin: 0; }

.error-box {
  background: rgba(180, 35, 24, 0.1);
  color: var(--warning);
  padding: 12px 14px;
  border-radius: 10px;
  border: 1px solid rgba(180, 35, 24, 0.2);
  font-size: 0.9rem;
}

.parse-btn { padding: 14px; font-size: 1rem; margin-top: auto; }

.btn-primary {
  background-color: var(--accent);
  color: #fff;
  border: none;
  border-radius: 12px;
  font-weight: 700;
  cursor: pointer;
  transition: 0.2s;
}

.btn-primary:hover:not(:disabled) { background-color: #0a6b5e; }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }

@media (min-width: 960px) {
  .parser-container { padding: 40px 32px 60px; }
  .editor-layout { flex-direction: row; gap: 28px; }
}
</style>