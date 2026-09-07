<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useCoverLettersStore, type CoverLetterDetail } from '../store/cover_letters';
import { useDialogStore } from '../store/dialog';
import { Motion, AnimatePresence } from 'motion-v';
import { invoke } from '@tauri-apps/api/core';
import { 
  ArrowLeft, 
  Edit, 
  Trash2, 
  X, 
  Save, 
  RotateCw,
  Copy,
  Check
} from '@lucide/vue';
import { copyToClipboard } from '../utils/clipboard';

// Codemirror imports
import { Codemirror } from 'vue-codemirror';
import { latex, latexLanguage, autoCloseTags } from 'codemirror-lang-latex';
import { oneDark } from '@codemirror/theme-one-dark';
import { EditorView } from '@codemirror/view';

const router = useRouter();
const clStore = useCoverLettersStore();
const dialog = useDialogStore();

// Codemirror Extensions
const extensions = [
  latex(),
  latexLanguage,
  ...autoCloseTags,
  oneDark,
  EditorView.lineWrapping
];

const props = defineProps<{ id: string }>();

interface UsageRecord {
  job_id: string;
  company_name: string;
  job_title: string;
}

// Tooltip State
const activeTooltip = ref<string | null>(null);

const isLoading = ref(true);
const isEditing = ref(false);
const isSaving = ref(false);
const isDeleting = ref(false);
const error = ref<string | null>(null);
const isErrorCopied = ref(false);
const handleCopyError = async () => {
  if (!error.value) return;
  const ok = await copyToClipboard(error.value);
  if (ok) {
    isErrorCopied.value = true;
    setTimeout(() => { isErrorCopied.value = false; }, 2000);
  }
};

const cl = ref<CoverLetterDetail | null>(null);
const editedName = ref('');
const editedCategory = ref('');
const editedLatex = ref('');

onMounted(async () => {
  try {
    cl.value = await clStore.getCoverLetterById(props.id);
    editedName.value = cl.value.name;
    editedCategory.value = cl.value.category;
    editedLatex.value = cl.value.latex_content;
  } catch (err: any) {
    error.value = err.toString();
  } finally {
    isLoading.value = false;
  }
});

const goBack = () => {
  if (window.history.length > 1) {
    router.back();
  } else {
    router.push('/templates/cover-letters');
  }
};

const toggleEditMode = async () => {
  if (isEditing.value) {
    const hasChanges =
      editedName.value !== (cl.value?.name || '') ||
      editedCategory.value !== (cl.value?.category || '') ||
      editedLatex.value !== (cl.value?.latex_content || '');

    if (hasChanges) {
      const confirmed = await dialog.showConfirm(
        'Are you sure you want to discard your unsaved changes?',
        'Discard Changes'
      );
      if (!confirmed) return;
    }

    editedName.value = cl.value?.name || '';
    editedCategory.value = cl.value?.category || '';
    editedLatex.value = cl.value?.latex_content || '';
    isEditing.value = false;
    await dialog.showAlert('Editing cancelled.', 'Cancelled');
    return;
  }
  isEditing.value = true;
};

const handleSave = async () => {
  if (!cl.value || !editedName.value.trim() || !editedCategory.value.trim()) {
    error.value = 'Name and category are required';
    await dialog.showAlert('Name and category are required to save the template.', 'Input Required');
    return;
  }

  isSaving.value = true;
  error.value = null;

  try {
    await clStore.updateCoverLetter(
      cl.value.id,
      editedName.value,
      editedCategory.value,
      editedLatex.value
    );

    const updated = await clStore.getCoverLetterById(props.id);
    cl.value = updated;
    isEditing.value = false;
    await dialog.showAlert('Template saved successfully.', 'Success');
  } catch (err: any) {
    error.value = err.toString();
    await dialog.showAlert(`Failed to save template: ${err.message || err.toString()}`, 'Save Error');
  } finally {
    isSaving.value = false;
  }
};

const handleDelete = async () => {
  if (!cl.value) return;

  try {
    // 1. Check for usage in tailored cover letters
    const usages = await invoke<UsageRecord[]>('check_cl_usage', { clId: cl.value.id });

    if (usages.length > 0) {
      const jobList = usages.map(u => `• ${u.company_name} (${u.job_title})`).join('\n');
      await dialog.showAlert(
        `This template cannot be deleted because it is currently used by tailored cover letters for the following jobs:\n\n${jobList}\n\nPlease delete these tailored versions or the jobs themselves before deleting this base template.`,
        'Template In Use'
      );
      return;
    }

    // 2. Proceed with normal confirmation
    const confirmed = await dialog.showConfirm('Delete this cover letter template? This cannot be undone.', 'Confirm Deletion');
    if (!confirmed) return;

    isDeleting.value = true;
    error.value = null;

    await clStore.deleteCoverLetter(cl.value.id);
    await dialog.showAlert('Template deleted successfully.', 'Success');
    router.push('/cover-letters');
  } catch (err: any) {
    error.value = err.toString();
    await dialog.showAlert(`Failed to delete template: ${err.message || err.toString()}`, 'Error');
  } finally {
    isDeleting.value = false;
  }
};

const hasLatexContent = () => {
  const content = cl.value?.latex_content || '';
  return content.trim().length > 0;
};
</script>

<template>
  <div class="detail-container" v-if="!isLoading">
    <header class="detail-header">
      <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'back'" @mouseleave="activeTooltip = null">
        <button class="back-btn" @click="goBack"><ArrowLeft :size="16" /></button>
        <AnimatePresence>
          <Motion
            v-if="activeTooltip === 'back'"
            :initial="{ opacity: 0, y: 5, scale: 0.9 }"
            :animate="{ opacity: 1, y: 0, scale: 1 }"
            :exit="{ opacity: 0, y: 5, scale: 0.9 }"
            :transition="{ duration: 0.15 }"
            class="flying-message header-tooltip"
          >
            Back to Templates
          </Motion>
        </AnimatePresence>
      </div>
      
      <div class="header-main">
        <div class="title-group" v-if="!isEditing">
          <h1>{{ cl?.name }}</h1>
          <span class="category-tag">{{ cl?.category }}</span>
        </div>
        <div class="edit-group" v-else>
          <input v-model="editedName" class="edit-input name-input" placeholder="Template Name" />
          <input v-model="editedCategory" class="edit-input category-input" placeholder="Category" />
        </div>
      </div>

      <div class="header-actions" v-if="!isEditing">
        <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'edit-tpl'" @mouseleave="activeTooltip = null">
          <button class="action-btn" @click="toggleEditMode"><Edit :size="16" /></button>
          <AnimatePresence>
            <Motion
              v-if="activeTooltip === 'edit-tpl'"
              :initial="{ opacity: 0, y: 5, scale: 0.9 }"
              :animate="{ opacity: 1, y: 0, scale: 1 }"
              :exit="{ opacity: 0, y: 5, scale: 0.9 }"
              :transition="{ duration: 0.15 }"
              class="flying-message header-tooltip"
            >
              Edit Template
            </Motion>
          </AnimatePresence>
        </div>
        <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'delete-tpl'" @mouseleave="activeTooltip = null">
          <button class="action-btn delete-btn" @click="handleDelete" :disabled="isDeleting">
            <RotateCw v-if="isDeleting" :size="16" class="spinner" />
            <Trash2 v-else :size="16" />
          </button>
          <AnimatePresence>
            <Motion
              v-if="activeTooltip === 'delete-tpl'"
              :initial="{ opacity: 0, y: 5, scale: 0.9 }"
              :animate="{ opacity: 1, y: 0, scale: 1 }"
              :exit="{ opacity: 0, y: 5, scale: 0.9 }"
              :transition="{ duration: 0.15 }"
              class="flying-message header-tooltip delete-tooltip"
            >
              Delete Template
            </Motion>
          </AnimatePresence>
        </div>
      </div>
    </header>

    <div class="content-wrapper">
      <div v-if="error" class="error-banner">
        <span>{{ error }}</span>
        <div class="banner-actions">
          <button class="banner-copy-btn" @click="handleCopyError" :title="isErrorCopied ? 'Copied!' : 'Copy Error'">
            <Check v-if="isErrorCopied" :size="13" />
            <Copy v-else :size="13" />
            <span>{{ isErrorCopied ? 'Copied!' : 'Copy' }}</span>
          </button>
          <button class="banner-close-btn" @click="error = null" title="Dismiss">✕</button>
        </div>
      </div>

      <div class="latex-section">
        <div class="section-header">
          <h2>LATEX SOURCE</h2>
          <div class="editor-actions">
            <span class="status-indicator" v-if="isEditing">Editing Mode</span>
          </div>
        </div>
        
        <codemirror
          v-if="isEditing"
          v-model="editedLatex"
          placeholder="Enter your LaTeX code here..."
          :style="{ minHeight: '400px', height: 'auto' }"
          :autofocus="true"
          :indent-with-tab="true"
          :tab-size="2"
          :extensions="extensions"
          class="latex-editor-cm"
        />

        <div v-if="isEditing" class="editor-bottom-bar">
          <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'cancel-edit'" @mouseleave="activeTooltip = null">
            <button type="button" class="action-btn cancel-btn" @click="toggleEditMode">
              <X :size="16" />
            </button>
            <AnimatePresence>
              <Motion
                v-if="activeTooltip === 'cancel-edit'"
                :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                :animate="{ opacity: 1, y: 0, scale: 1 }"
                :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                :transition="{ duration: 0.15 }"
                class="flying-message"
              >
                Cancel
              </Motion>
            </AnimatePresence>
          </div>

          <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'save-edit'" @mouseleave="activeTooltip = null">
            <button type="button" class="action-btn save-btn" @click="handleSave" :disabled="isSaving">
              <RotateCw v-if="isSaving" :size="16" class="spinner" />
              <Save v-else :size="16" />
            </button>
            <AnimatePresence>
              <Motion
                v-if="activeTooltip === 'save-edit'"
                :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                :animate="{ opacity: 1, y: 0, scale: 1 }"
                :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                :transition="{ duration: 0.15 }"
                class="flying-message"
              >
                {{ isSaving ? 'Saving...' : 'Save Changes' }}
              </Motion>
            </AnimatePresence>
          </div>
        </div>
        <div v-else-if="hasLatexContent()" class="latex-preview">
          <pre><code>{{ cl?.latex_content }}</code></pre>
        </div>
        <div v-else class="empty-latex">
          <p>This template has no LaTeX content yet.</p>
          <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'add-latex'" @mouseleave="activeTooltip = null">
            <button class="btn-edit" @click="toggleEditMode"><Edit :size="16" /></button>
            <AnimatePresence>
              <Motion
                v-if="activeTooltip === 'add-latex'"
                :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                :animate="{ opacity: 1, y: 0, scale: 1 }"
                :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                :transition="{ duration: 0.15 }"
                class="flying-message"
              >
                Add LaTeX
              </Motion>
            </AnimatePresence>
          </div>
        </div>
      </div>

      <div class="meta-info" v-if="!isEditing">
        <div class="meta-item">
          <label>CREATED</label>
          <span>{{ new Date(cl?.created_at || '').toLocaleString() }}</span>
        </div>
        <div class="meta-item">
          <label>LAST UPDATED</label>
          <span>{{ new Date(cl?.updated_at || '').toLocaleString() }}</span>
        </div>
      </div>
    </div>
  </div>
  <div class="loading" v-else>
    Loading cover letter details...
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
  height: 56px;
  display: flex;
  align-items: center;
  padding: 0 24px;
  background: var(--bg-accent);
  border-bottom: 1px solid var(--line);
  gap: 20px;
}

.back-btn {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  padding: 8px;
  min-width: 38px;
  min-height: 38px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: 0.2s;
  flex-shrink: 0;
}
.back-btn:hover { background: var(--surface); color: var(--ink); }

.header-main { flex: 1; min-width: 0; }
.title-group { display: flex; align-items: center; gap: 12px; min-width: 0; }
.title-group h1 { font-size: 1.1rem; font-weight: 700; color: var(--ink); margin: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; min-width: 0; }

.category-tag {
  background: var(--surface);
  color: var(--muted);
  font-size: 0.6rem;
  font-weight: 800;
  padding: 2px 8px;
  border-radius: 4px;
  border: 1px solid var(--line);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  white-space: nowrap;
  flex-shrink: 0;
}

.edit-group { display: flex; gap: 12px; }
.edit-input {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 6px;
  color: var(--ink);
  padding: 6px 12px;
  min-height: 38px;
  font-size: 0.9rem;
  outline: none;
  box-sizing: border-box;
}
.name-input { font-weight: 700; flex: 1; min-width: 0; }
.category-input { width: fit-content; min-width: 120px; }

.edit-input:focus {
  border-color: var(--accent);
}

.header-actions {
  display: flex;
  gap: 10px;
  align-items: center;
  flex-shrink: 0;
}

.action-btn {
  background: var(--surface);
  border: 1px solid var(--line);
  color: var(--muted);
  width: 38px;
  height: 38px;
  min-width: 38px;
  min-height: 38px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  cursor: pointer;
  transition: 0.2s;
  flex-shrink: 0;
}
.action-btn:hover:not(:disabled) { border-color: var(--accent); color: var(--accent); }
.action-btn:disabled { opacity: 0.5; cursor: not-allowed; }

.delete-btn:hover:not(:disabled) { border-color: var(--warning); color: var(--warning); }
.cancel-btn:hover:not(:disabled) { border-color: var(--muted); color: var(--ink); }
.save-btn { background: var(--accent); color: white; border: none; }
.save-btn:hover:not(:disabled) { background: var(--accent-hover); color: white; }

.content-wrapper {
  flex: 1;
  overflow-y: auto;
  padding: 32px 24px;
  max-width: 1000px;
  width: 100%;
  margin: 0 auto;
  box-sizing: border-box;
}

/* Content wrapper scrollbar: 2 units default to 5 units on focus with 18px clearance */
.content-wrapper::-webkit-scrollbar {
  width: 4px;
  height: 2px;
  transition: all 0.15s ease;
}

.content-wrapper:hover::-webkit-scrollbar,
.content-wrapper:focus-within::-webkit-scrollbar {
  width: 6px;
  height: 5px;
}

.content-wrapper::-webkit-scrollbar-track {
  background: transparent;
  margin: 18px 0;
}

.content-wrapper::-webkit-scrollbar-thumb {
  background: var(--line);
  border-radius: 4px;
}

.content-wrapper::-webkit-scrollbar-thumb:hover {
  background: var(--muted);
}

.error-banner {
  background: var(--surface-soft);
  color: var(--warning);
  border: 1px solid var(--warning);
  padding: 10px 16px;
  border-radius: 8px;
  font-size: 0.85rem;
  margin-bottom: 24px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  word-break: break-word;
}

.banner-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.banner-copy-btn {
  background: var(--surface);
  border: 1px solid var(--line);
  color: var(--ink);
  padding: 6px 10px;
  min-height: 32px;
  border-radius: 6px;
  font-size: 0.75rem;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  transition: all 0.2s;
}

.banner-copy-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
}

.banner-close-btn {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  padding: 6px;
  min-width: 32px;
  min-height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.banner-close-btn:hover {
  color: var(--ink);
}

.latex-section {
  margin-bottom: 32px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.section-header h2 {
  font-size: 0.7rem;
  font-weight: 900;
  color: var(--muted);
  letter-spacing: 0.1em;
  margin: 0;
}

.status-indicator {
  font-size: 0.65rem;
  color: var(--accent);
  font-weight: 700;
}

.latex-editor-cm {
  width: 100%;
  max-width: 100%;
  box-sizing: border-box;
  background-color: #282c34; /* One Dark background */
  border: 1px solid var(--line);
  border-radius: 12px;
  overflow: hidden;
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 0.9rem;
}

:deep(.cm-editor) {
  outline: none !important;
}

:deep(.cm-scroller) {
  font-family: inherit;
}

:deep(.cm-content) {
  padding: 16px 0;
}

:deep(.cm-gutters) {
  background-color: #282c34 !important;
  border-right: 1px solid #3e4451 !important;
  color: #abb2bf !important;
}

.editor-bottom-bar {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 12px;
  margin-top: 16px;
  padding-top: 14px;
  border-top: 1px solid var(--line);
}

.latex-preview {
  width: 100%;
  box-sizing: border-box;
  min-height: 280px;
  max-height: min(520px, 60vh);
  background-color: var(--surface);
  border: 1px solid var(--line);
  border-radius: 12px;
  padding: 16px;
  color: var(--ink);
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 0.8rem;
  line-height: 1.6;
  overflow: auto;
}

/* Latex preview scrollbar track clearance: 2 units default to 5 units on focus */
.latex-preview::-webkit-scrollbar {
  width: 4px;
  height: 2px;
  transition: all 0.15s ease;
}

.latex-preview:hover::-webkit-scrollbar,
.latex-preview:focus-within::-webkit-scrollbar {
  width: 6px;
  height: 5px;
}

.latex-preview::-webkit-scrollbar-track {
  background: transparent;
  margin: 8px;
}

.latex-preview::-webkit-scrollbar-thumb {
  background: var(--line);
  border-radius: 4px;
}

.latex-preview pre { margin: 0; white-space: pre-wrap; }

.empty-latex {
  width: 100%;
  box-sizing: border-box;
  min-height: 220px;
  background-color: var(--surface);
  border: 1px dashed var(--line);
  border-radius: 12px;
  padding: 20px;
  color: var(--muted);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
}
.empty-latex p { margin: 0; font-size: 0.9rem; }

.btn-edit {
  background: var(--accent);
  color: white;
  border: none;
  padding: 8px 16px;
  min-height: 38px;
  border-radius: 8px;
  cursor: pointer;
}

.meta-info {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
  padding-top: 24px;
  border-top: 1px solid var(--line);
}

.meta-item label {
  display: block;
  font-size: 0.6rem;
  font-weight: 800;
  color: var(--muted);
  margin-bottom: 4px;
}

.meta-item span {
  font-size: 0.85rem;
  color: var(--ink);
}

.btn-tooltip-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.flying-message {
  position: absolute;
  bottom: 140%;
  left: 50%;
  transform: translateX(-50%);
  background: var(--accent);
  color: white;
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 0.65rem;
  font-weight: 700;
  white-space: nowrap;
  pointer-events: none;
  z-index: 1000;
  box-shadow: 0 4px 12px rgba(0,0,0,0.3);
}

.flying-message::after {
  content: '';
  position: absolute;
  top: 100%;
  left: 50%;
  transform: translateX(-50%);
  border: 4px solid transparent;
  border-top-color: var(--accent);
}

.header-tooltip { bottom: auto; top: 140%; }
.header-tooltip::after { top: auto; bottom: 100%; border-top-color: transparent; border-bottom-color: var(--accent); }
.delete-tooltip { background: var(--warning); left: auto; right: 0; transform: none; }
.delete-tooltip::after { border-bottom-color: var(--warning); left: auto; right: 8px; transform: none; }

.loading {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--muted);
}

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* Responsive Breakpoints */
@media (max-width: 959px) {
  .detail-header {
    padding: 0 16px;
    gap: 14px;
  }
  .content-wrapper {
    padding: 24px 16px;
  }
}

@media (max-width: 600px) {
  .detail-header {
    height: auto;
    min-height: 52px;
    padding: 10px 12px;
    gap: 10px;
    flex-wrap: wrap;
  }
  .title-group h1 {
    font-size: 0.95rem;
  }
  .category-tag {
    font-size: 0.58rem;
  }
  .edit-group {
    flex-direction: column;
    gap: 6px;
    width: 100%;
  }
  .name-input,
  .category-input {
    width: 100%;
  }
  .action-btn {
    width: 40px;
    height: 40px;
    min-width: 40px;
    min-height: 40px;
  }
  .back-btn {
    width: 40px;
    height: 40px;
    min-width: 40px;
    min-height: 40px;
  }
  .content-wrapper {
    padding: 16px 12px;
  }
  .latex-editor-cm {
    font-size: 0.82rem;
  }
  .latex-preview {
    padding: 12px 10px;
    font-size: 0.76rem;
  }
  .meta-info {
    grid-template-columns: 1fr;
    gap: 12px;
    padding-top: 16px;
  }
  .editor-bottom-bar {
    justify-content: flex-end;
    gap: 10px;
  }
  .error-banner {
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }
  .banner-actions {
    align-self: flex-end;
  }
}

@media (max-width: 360px) {
  .detail-header {
    padding: 8px;
    gap: 8px;
  }
  .content-wrapper {
    padding: 12px 8px;
  }
  .title-group h1 {
    font-size: 0.88rem;
  }
  .latex-preview {
    font-size: 0.72rem;
  }
  .editor-bottom-bar {
    justify-content: flex-end;
    gap: 8px;
  }
}
</style>
