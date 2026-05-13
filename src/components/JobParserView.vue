<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useJobsStore } from '../store/jobs';

const router = useRouter();
const jobsStore = useJobsStore();
const rawJobDescription = ref('');

const handleParse = async () => {
  if (!rawJobDescription.value.trim()) return;
  
  try {
    const slug = await jobsStore.parseNewJob(rawJobDescription.value);
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
.parser-container { padding: 40px; height: 100%; display: flex; flex-direction: column; }
.header { display: flex; align-items: center; gap: 20px; margin-bottom: 30px; }
.header h2 { margin: 0; font-weight: 600; color: #ededed; }
.back-btn { background: transparent; border: 1px solid rgba(255,255,255,0.1); color: #888; padding: 8px 16px; border-radius: 6px; cursor: pointer; transition: 0.2s; }
.back-btn:hover { color: #ededed; border-color: rgba(255,255,255,0.3); }

.editor-layout { display: flex; gap: 30px; flex-grow: 1; min-height: 0; }
.input-section { flex: 2; display: flex; flex-direction: column; }
label { color: #00e599; font-weight: 600; margin-bottom: 10px; font-size: 0.9rem; }
textarea { flex-grow: 1; background: #0a0a0a; border: 1px solid rgba(255,255,255,0.1); border-radius: 12px; padding: 20px; color: #ededed; font-family: monospace; resize: none; outline: none; transition: 0.2s; }
textarea:focus { border-color: #00e599; box-shadow: 0 0 0 1px rgba(0,229,153,0.3); }

.action-section { flex: 1; display: flex; flex-direction: column; gap: 20px; }
.info-card { background: rgba(255,255,255,0.02); border: 1px solid rgba(255,255,255,0.05); padding: 24px; border-radius: 12px; }
.info-card h3 { color: #ededed; margin: 0 0 10px 0; font-size: 1.1rem; }
.info-card p { color: #888; font-size: 0.9rem; line-height: 1.5; margin: 0; }

.error-box { background: rgba(255, 50, 50, 0.1); color: #ff5555; padding: 15px; border-radius: 8px; border: 1px solid rgba(255, 50, 50, 0.2); font-size: 0.9rem; }

.parse-btn { padding: 16px; font-size: 1.1rem; margin-top: auto; }
.btn-primary { background-color: #00e599; color: #000; border: none; border-radius: 8px; font-weight: 600; cursor: pointer; transition: 0.2s; }
.btn-primary:hover:not(:disabled) { background-color: #00c785; }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
</style>