<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { InboxJob } from '../store/inbox';
import { useSettingsStore } from '../store/settings';
import { useDialogStore } from '../store/dialog';
import { 
  ArrowLeft, 
  ExternalLink, 
  Cpu, 
  Trash2, 
  Clock, 
  RefreshCw,
  Globe,
  FileText,
  Copy,
  Check,
  Hash,
  Type
} from '@lucide/vue';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';

const route = useRoute();
const router = useRouter();
const settingsStore = useSettingsStore();
const dialog = useDialogStore();

const job = ref<InboxJob | null>(null);
const isLoading = ref(true);
const isProcessing = ref(false);
const isCopied = ref(false);

const stats = computed(() => {
  if (!job.value) return { chars: 0, words: 0 };
  const text = job.value.raw_description || '';
  return {
    chars: text.length,
    words: text.split(/\s+/).filter(w => w.length > 0).length
  };
});

const loadJob = async () => {
  isLoading.value = true;
  try {
    const id = route.params.id as string;
    job.value = await invoke<InboxJob>('get_inbox_job_by_id', { id });
  } catch (err: any) {
    console.error('Failed to load inbox job:', err);
    await dialog.showAlert('Failed to load job details.', 'Error');
    router.push('/inbox');
  } finally {
    isLoading.value = false;
  }
};

onMounted(loadJob);

const goBack = () => router.push('/inbox');

const copyToClipboard = async () => {
  if (!job.value) return;
  await writeText(job.value.raw_description);
  isCopied.value = true;
  setTimeout(() => isCopied.value = false, 2000);
};

const processJob = async () => {
  if (!job.value || isProcessing.value) return;
  
  isProcessing.value = true;
  try {
    const apiKey = await settingsStore.getDecryptedKey();
    if (!apiKey) {
      await dialog.showAlert('Please set your AI API key in Settings first.', 'API Key Missing');
      return;
    }

    const result = await invoke<any>('parse_job', {
      provider: settingsStore.selectedAiProvider,
      model: settingsStore.selectedAiModel,
      apiKey,
      rawJd: job.value.raw_description,
      jobUrl: job.value.url
    });

    const jobPayload = {
      id: Math.random().toString(36).substring(2, 11),
      company_name: result.details.company_name,
      job_title: result.details.job_title,
      work_model: result.details.work_model,
      employment_type: result.details.employment_type,
      status: 'Drafting',
      raw_jd: job.value.raw_description,
      requirements: JSON.stringify(result.details.requirements || []),
      core_responsibilities: JSON.stringify(result.details.core_responsibilities || []),
      job_url: job.value.url,
    };

    await invoke('save_job', { payload: jobPayload });
    await invoke('mark_inbox_job_processed', { id: job.value.id });
    
    await dialog.showAlert('Job processed and moved to vault successfully!', 'Success');
    router.push('/inbox');
  } catch (error: any) {
    console.error('Processing error:', error);
    await dialog.showAlert(`Failed to process job: ${error.toString()}`, 'Error');
  } finally {
    isProcessing.value = false;
  }
};

const deleteJob = async () => {
  if (!job.value) return;
  const confirmed = await dialog.showConfirm('Are you sure you want to delete this captured job data?', 'Delete Capture');
  if (confirmed) {
    try {
      await invoke('delete_inbox_job', { id: job.value.id });
      router.push('/inbox');
    } catch (err: any) {
      await dialog.showAlert(err.toString(), 'Delete Failed');
    }
  }
};
</script>

<template>
  <div class="detail-container">
    <header class="detail-header">
      <button class="back-btn" @click="goBack" title="Back to Inbox" aria-label="Back to Inbox">
        <ArrowLeft :size="18" />
      </button>
      <div class="header-main">
        <h1>Capture Details</h1>
        <div class="badge-row" v-if="job">
          <span :class="['status-badge', job.status.toLowerCase()]">{{ job.status }}</span>
          <span class="timestamp"><Clock :size="12" /> {{ job.created_at }}</span>
        </div>
      </div>
      <div class="header-actions" v-if="job">
        <button class="action-btn danger" @click="deleteJob" title="Delete Capture" aria-label="Delete Capture">
          <Trash2 :size="16" />
          <span class="btn-text">Delete</span>
        </button>
        <button 
          v-if="job.status === 'Pending'"
          class="action-btn primary" 
          @click="processJob"
          :disabled="isProcessing"
          title="Process with AI"
          aria-label="Process with AI"
        >
          <RefreshCw v-if="isProcessing" :size="16" class="spinner" />
          <Cpu v-else :size="16" />
          <span class="btn-text">{{ isProcessing ? 'Processing...' : 'Process with AI' }}</span>
        </button>
      </div>
    </header>

    <main class="detail-content">
      <div v-if="isLoading" class="loading-state">
        <RefreshCw :size="48" class="spinner" />
        <p>Loading capture data...</p>
      </div>

      <template v-else-if="job">
        <section class="info-section">
          <div class="section-card">
            <div class="card-header">
              <Globe :size="18" />
              <h3>Source URL</h3>
            </div>
            <div class="url-box">
              <a v-if="job.url" :href="job.url" target="_blank" class="job-link">
                {{ job.url }}
                <ExternalLink :size="14" />
              </a>
              <span v-else class="no-url">No URL captured</span>
            </div>
          </div>

          <div class="section-card description-card">
            <div class="card-header">
              <div class="header-left">
                <FileText :size="18" />
                <h3>Raw Content</h3>
              </div>
              <div class="header-right">
                <div class="stat-item">
                  <Hash :size="12" />
                  <span>{{ stats.chars.toLocaleString() }} chars</span>
                </div>
                <div class="stat-divider"></div>
                <div class="stat-item">
                  <Type :size="12" />
                  <span>{{ stats.words.toLocaleString() }} words</span>
                </div>
                <button class="copy-small-btn" @click="copyToClipboard" :title="isCopied ? 'Copied!' : 'Copy to clipboard'">
                  <component :is="isCopied ? Check : Copy" :size="14" />
                </button>
              </div>
            </div>
            <div class="content-box">
              <pre class="raw-text">{{ job.raw_description }}</pre>
            </div>
          </div>
        </section>
      </template>
    </main>
  </div>
</template>

<style scoped>
.detail-container {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg);
}

.detail-header {
  height: 72px;
  padding: 0 40px;
  display: flex;
  align-items: center;
  gap: 20px;
  background: var(--bg-accent);
  border-bottom: 1px solid var(--line);
  flex-shrink: 0;
}

.back-btn {
  background: var(--surface-soft);
  border: 1px solid var(--line);
  color: var(--ink);
  width: 42px;
  height: 42px;
  min-width: 42px;
  min-height: 42px;
  border-radius: var(--radius-md, 8px);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.back-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
  background: var(--bg);
}

.back-btn:active {
  transform: scale(0.95);
}

.header-main {
  flex: 1;
  min-width: 0;
}

.header-main h1 {
  margin: 0;
  font-size: 1.4rem;
  color: var(--ink);
  font-weight: 700;
  line-height: 1.2;
}

.badge-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 4px;
  flex-wrap: wrap;
}

.status-badge {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 0.65rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.status-badge.pending {
  background: var(--accent-soft);
  color: var(--accent);
  border: 1px solid var(--accent);
}

.status-badge.processed {
  background: var(--surface-soft);
  color: var(--muted);
  border: 1px solid var(--line);
}

.timestamp {
  font-size: 0.72rem;
  color: var(--muted);
  font-family: monospace;
  display: flex;
  align-items: center;
  gap: 4px;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}

.action-btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 0 16px;
  height: 42px;
  min-height: 42px;
  border-radius: var(--radius-md, 8px);
  font-weight: 600;
  font-size: 0.85rem;
  cursor: pointer;
  transition: all 0.15s ease;
  border: 1px solid var(--line);
  white-space: nowrap;
}

.action-btn:active:not(:disabled) {
  transform: scale(0.95);
}

.action-btn.primary {
  background: var(--accent);
  color: white;
  border-color: var(--accent);
}

.action-btn.primary:hover:not(:disabled) {
  background: var(--accent-hover, #2ea043);
  transform: translateY(-1px);
}

.action-btn.danger {
  background: transparent;
  color: var(--warning);
  border-color: var(--warning);
}

.action-btn.danger:hover:not(:disabled) {
  background: rgba(248, 81, 73, 0.12);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.detail-content {
  flex: 1;
  overflow-y: auto;
  padding: 32px 40px;
  scrollbar-width: thin;
  scrollbar-color: var(--line) transparent;
}

.detail-content::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

.detail-content::-webkit-scrollbar-track {
  background: transparent;
  margin: 6px 0;
}

.detail-content::-webkit-scrollbar-thumb {
  background: var(--line);
  border-radius: 4px;
}

.detail-content::-webkit-scrollbar-thumb:hover {
  background: var(--accent);
}

.info-section {
  max-width: 1000px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.section-card {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: var(--radius-lg, 14px);
  padding: 22px;
  box-shadow: var(--shadow);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  color: var(--accent);
  margin-bottom: 16px;
  gap: 12px;
}

.card-header h3 {
  margin: 0;
  font-size: 1.05rem;
  font-weight: 600;
  color: var(--ink);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 0.72rem;
  color: var(--muted);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.stat-divider {
  width: 1px;
  height: 12px;
  background: var(--line);
}

.copy-small-btn {
  background: var(--surface-soft);
  border: 1px solid var(--line);
  color: var(--muted);
  width: 32px;
  height: 32px;
  min-width: 32px;
  min-height: 32px;
  border-radius: var(--radius-md, 6px);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
  margin-left: 4px;
}

.copy-small-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
  background: var(--surface);
}

.copy-small-btn:active {
  transform: scale(0.92);
}

.url-box {
  background: var(--bg);
  padding: 14px;
  border-radius: var(--radius-md, 8px);
  border: 1px solid var(--line);
}

.job-link {
  color: var(--accent);
  text-decoration: none;
  font-weight: 600;
  font-size: 0.85rem;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  word-break: break-all;
}

.job-link:hover {
  text-decoration: underline;
}

.no-url {
  color: var(--muted);
  font-style: italic;
  font-size: 0.85rem;
}

.description-card {
  flex: 1;
}

.content-box {
  background: var(--bg);
  border-radius: var(--radius-md, 8px);
  border: 1px solid var(--line);
  overflow-x: auto;
  overflow-y: hidden;
  padding-bottom: 4px;
  scrollbar-width: thin;
  scrollbar-color: var(--line) transparent;
}

.content-box::-webkit-scrollbar {
  height: 4px;
}

.content-box::-webkit-scrollbar-track {
  background: transparent;
  margin: 0 4px;
}

.content-box::-webkit-scrollbar-thumb {
  background: var(--line);
  border-radius: 4px;
}

.content-box::-webkit-scrollbar-thumb:hover {
  background: var(--accent);
}

.raw-text {
  margin: 0;
  padding: 20px;
  font-family: inherit;
  font-size: 0.9rem;
  line-height: 1.6;
  color: var(--ink);
  white-space: pre-wrap;
  word-break: break-word;
  user-select: text !important;
  -webkit-user-select: text !important;
}

.loading-state {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--muted);
  gap: 14px;
}

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* =======================================================================
   Tablet Styles (601px - 959px)
   ======================================================================= */
@media (max-width: 959px) and (min-width: 601px) {
  .detail-header {
    height: 64px;
    padding: 0 20px;
    gap: 16px;
  }

  .header-main h1 {
    font-size: 1.3rem;
  }

  .detail-content {
    padding: 20px 18px;
  }

  .section-card {
    padding: 18px;
    border-radius: 12px;
  }

  .raw-text {
    padding: 16px;
    font-size: 0.88rem;
  }
}

/* =======================================================================
   Mobile Styles (<= 600px):
   Touch targets min 38px, bounds safety, responsive wrapping
   ======================================================================= */
@media (max-width: 600px) {
  .detail-header {
    height: auto;
    min-height: 56px;
    padding: 12px 14px;
    gap: 10px;
    flex-wrap: wrap;
    justify-content: space-between;
  }

  .back-btn {
    width: 38px;
    height: 38px;
    min-width: 38px;
    min-height: 38px;
  }

  .header-main {
    flex: 1;
    min-width: 140px;
  }

  .header-main h1 {
    font-size: 1.2rem;
  }

  .header-actions {
    gap: 8px;
  }

  .action-btn {
    height: 38px;
    min-height: 38px;
    padding: 0 12px;
    font-size: 0.8rem;
    gap: 6px;
  }

  .detail-content {
    padding: 14px 12px;
  }

  .info-section {
    gap: 14px;
  }

  .section-card {
    padding: 14px 12px;
    border-radius: var(--radius-md, 8px);
  }

  .card-header {
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 12px;
  }

  .header-right {
    flex-wrap: wrap;
    gap: 6px;
  }

  .stat-item {
    font-size: 0.68rem;
  }

  .copy-small-btn {
    width: 34px;
    height: 34px;
    min-width: 34px;
    min-height: 34px;
  }

  .raw-text {
    padding: 14px 10px;
    font-size: 0.82rem;
  }
}

/* =======================================================================
   Ultra-compact Mobile (<= 340px):
   Icon-only action buttons and tighter padding for 300x400 screens
   ======================================================================= */
@media (max-width: 340px) {
  .detail-header {
    padding: 10px 8px;
    gap: 8px;
  }

  .header-main h1 {
    font-size: 1.05rem;
  }

  .action-btn .btn-text {
    display: none;
  }

  .action-btn {
    width: 38px;
    height: 38px;
    min-width: 38px;
    min-height: 38px;
    padding: 0;
    justify-content: center;
  }

  .detail-content {
    padding: 10px 6px;
  }
}
</style>
