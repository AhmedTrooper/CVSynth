<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useJobsStore } from '../store/jobs';
import { Motion } from 'motion-v';

const router = useRouter();
const jobsStore = useJobsStore();
const rawJobDescription = ref('');
const jobUrl = ref('');

const handleParse = async () => {
  if (!rawJobDescription.value.trim()) return;
  
  try {
    const slug = await jobsStore.parseNewJob(rawJobDescription.value, jobUrl.value);
    router.push(`/job/${slug}`);
  } catch (e) {
    console.error(e);
  }
};
</script>

<template>
  <div class="parser-container">
    <header class="header">
      <button class="back-btn" @click="router.push('/')">←</button>
      <h2>NEW APPLICATION</h2>
    </header>

    <div class="workspace">
      <div class="input-panel">
        <div class="field-group">
          <label>URL (OPTIONAL)</label>
          <input 
            v-model="jobUrl" 
            type="url" 
            placeholder="Link to job posting..."
            class="native-input"
          />
        </div>

        <div class="field-group expand">
          <label>RAW DESCRIPTION</label>
          <textarea 
            v-model="rawJobDescription" 
            placeholder="Paste description, requirements, etc..."
            spellcheck="false"
            class="native-textarea"
          ></textarea>
        </div>
      </div>

      <div class="side-panel">
        <div class="info-card">
          <h3>INTELLIGENCE</h3>
          <p>The AI will extract structured data to automate your resume tailoring.</p>
        </div>
        
        <div v-if="jobsStore.error" class="error-msg">
          {{ jobsStore.error }}
        </div>

        <button 
          class="btn-primary" 
          @click="handleParse" 
          :disabled="jobsStore.isLoading || !rawJobDescription"
        >
          <Motion
            v-if="jobsStore.isLoading"
            :animate="{ rotate: 360 }"
            :transition="{ repeat: Infinity, duration: 1, ease: 'linear' }"
            class="loader"
          >⚙️</Motion>
          <span v-else>RUN EXTRACTION →</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.parser-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--bg);
}

.header {
  height: 36px;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 12px;
  background: var(--bg-accent);
  border-bottom: 1px solid var(--line);
}

.header h2 { font-size: 0.65rem; color: var(--muted); margin: 0; letter-spacing: 0.05em; }

.back-btn { background: none; border: none; color: var(--muted); cursor: pointer; font-size: 1.2rem; padding: 0 4px; }
.back-btn:hover { color: var(--ink); }

.workspace {
  flex: 1;
  display: flex;
  min-height: 0;
}

.input-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 20px;
  gap: 20px;
}

.field-group { display: flex; flex-direction: column; gap: 8px; }
.field-group.expand { flex: 1; min-height: 0; }

label {
  font-size: 0.65rem;
  font-weight: 700;
  color: var(--muted);
  letter-spacing: 0.05em;
}

.native-input, .native-textarea {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: var(--radius-md);
  color: var(--ink);
  padding: 10px 12px;
  font-size: 0.85rem;
  outline: none;
  transition: 0.15s;
}
.native-input:focus, .native-textarea:focus { border-color: var(--accent); }

.native-textarea {
  flex: 1;
  resize: none;
  font-family: 'JetBrains Mono', monospace;
  line-height: 1.5;
}

.side-panel {
  width: 280px;
  background: var(--bg-accent);
  border-left: 1px solid var(--line);
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.info-card {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: var(--radius-md);
  padding: 12px;
}
.info-card h3 { font-size: 0.65rem; color: var(--accent); margin: 0 0 6px 0; }
.info-card p { font-size: 0.75rem; color: var(--muted); margin: 0; line-height: 1.4; }

.error-msg {
  font-size: 0.75rem;
  color: var(--warning);
  background: rgba(248, 81, 73, 0.1);
  padding: 8px;
  border-radius: var(--radius-sm);
}

.btn-primary {
  margin-top: auto;
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: var(--radius-md);
  padding: 10px;
  font-weight: 700;
  font-size: 0.75rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }

.loader { font-size: 1rem; }

@media (max-width: 960px) {
  .workspace { flex-direction: column; }
  .side-panel { width: 100%; border-left: none; border-top: 1px solid var(--line); }
}
</style>