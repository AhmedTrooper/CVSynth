<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { useOutreachStore, OutreachLead } from '../store/outreach';
import { useHrMessagesStore } from '../store/hr_messages';
import { useDialogStore } from '../store/dialog';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { openUrl } from '@tauri-apps/plugin-opener';
import { 
  Plus, 
  Send, 
  Search, 
  X, 
  Save, 
  RotateCw, 
  CheckSquare, 
  Square, 
  Trash2, 
  Copy, 
  Check, 
  ExternalLink, 
  Sparkles, 
  User, 
  Edit3,
  ChevronLeft,
  ChevronRight
} from '@lucide/vue';
import { Motion, AnimatePresence } from 'motion-v';
import CustomSelect from './CustomSelect.vue';

const outreachStore = useOutreachStore();
const hrStore = useHrMessagesStore();
const dialog = useDialogStore();

// Tooltip State
const activeTooltip = ref<string | null>(null);

// Selection State
const isSelectionMode = ref(false);
const selectedIds = ref<Set<string>>(new Set());

// Search & Filter State
const searchQuery = ref('');
const statusFilter = ref<string>('ALL');

// Form & Editor State
const showForm = ref(false);
const editingLeadId = ref<string | null>(null);

const formName = ref('');
const formProfileUrl = ref('');
const formHeadline = ref('');
const formRawBio = ref('');
const formPosts = ref<string[]>(['', '']); // default 2 post slots
const formTemplateId = ref<string>('');
const formCharLimit = ref<number>(200);
const formCustomInstruction = ref('');
const formTailoredMessage = ref('');
const formStatus = ref<string>('Draft');

const isGenerating = ref(false);
const isSaving = ref(false);
const copiedId = ref<string | null>(null);
const formCopied = ref(false);

// Character Limit Presets
const PRESETS = [200, 250, 300, 500];

// Themed dropdown options (CustomSelect renders its own popup, so the
// option list follows the global theme instead of native OS styling)
const templateOptions = computed(() => [
  { value: '', label: '-- No base template (Tailor from scratch) --' },
  ...hrStore.templates.map(t => ({ value: t.id, label: `${t.name} (${t.category})` }))
]);

const statusOptions = [
  { value: 'Draft', label: 'Draft' },
  { value: 'Sent', label: 'Sent' },
  { value: 'Connected', label: 'Connected' },
  { value: 'Replied', label: 'Replied' },
  { value: 'Archived', label: 'Archived' }
];

onMounted(async () => {
  await Promise.all([
    outreachStore.loadAllLeads(),
    hrStore.loadTemplates()
  ]);
});

// Pagination State
const currentPage = ref(1);
const pageSize = ref(20);
const pageSizeOptions = [
  { value: 10, label: '10 per page' },
  { value: 20, label: '20 per page' },
  { value: 50, label: '50 per page' },
  { value: 100, label: '100 per page' },
];

watch([searchQuery, statusFilter, pageSize], () => {
  currentPage.value = 1;
});

// Filtered Leads
const filteredLeads = computed(() => {
  const rawList = outreachStore.leads;
  if (!Array.isArray(rawList)) return [];
  let list = rawList.filter(l => l && typeof l === 'object' && typeof l.id === 'string');

  if (statusFilter.value !== 'ALL') {
    list = list.filter(l => l.status === statusFilter.value);
  }

  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase().trim();
    list = list.filter(l => 
      (l.person_name || '').toLowerCase().includes(q) ||
      (l.headline && l.headline.toLowerCase().includes(q)) ||
      (l.raw_bio || '').toLowerCase().includes(q) ||
      (l.tailored_message && l.tailored_message.toLowerCase().includes(q))
    );
  }

  return list;
});

const totalPages = computed(() => {
  return Math.max(1, Math.ceil(filteredLeads.value.length / pageSize.value));
});

watch(totalPages, (newTotal) => {
  if (currentPage.value > newTotal) {
    currentPage.value = Math.max(1, newTotal);
  }
});

const paginatedLeads = computed(() => {
  if (!Array.isArray(filteredLeads.value)) return [];
  const start = (currentPage.value - 1) * pageSize.value;
  return filteredLeads.value.slice(start, start + pageSize.value);
});

// Selection helpers
const toggleSelection = (id: string) => {
  if (selectedIds.value.has(id)) {
    selectedIds.value.delete(id);
  } else {
    selectedIds.value.add(id);
  }
};

const toggleSelectAll = () => {
  if (selectedIds.value.size === paginatedLeads.value.length) {
    selectedIds.value.clear();
  } else {
    selectedIds.value = new Set(paginatedLeads.value.map(l => l.id));
  }
};

const exitSelectionMode = () => {
  isSelectionMode.value = false;
  selectedIds.value.clear();
};

const handleBatchDelete = async () => {
  if (selectedIds.value.size === 0) return;

  const count = selectedIds.value.size;
  const confirmed = await dialog.showConfirm(
    `Are you sure you want to delete ${count} outreach lead(s)? This action cannot be undone.`,
    'Delete Leads'
  );

  if (confirmed) {
    try {
      await outreachStore.deleteBatch(Array.from(selectedIds.value));
      selectedIds.value.clear();
      await dialog.showAlert('Leads deleted successfully.', 'Success');
    } catch (err: any) {
      console.error(err);
      await dialog.showAlert('Failed to delete selected leads.', 'Error');
    }
  }
};

// Form Management
const resetForm = () => {
  editingLeadId.value = null;
  formName.value = '';
  formProfileUrl.value = '';
  formHeadline.value = '';
  formRawBio.value = '';
  formPosts.value = ['', ''];
  formTemplateId.value = '';
  formCharLimit.value = 200;
  formCustomInstruction.value = '';
  formTailoredMessage.value = '';
  formStatus.value = 'Draft';
  formCopied.value = false;
};

const openNewForm = () => {
  resetForm();
  showForm.value = true;
};

const closeForm = () => {
  if (isSaving.value || isGenerating.value) return;
  showForm.value = false;
  resetForm();
};

const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Escape' && showForm.value) {
    closeForm();
  }
};

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});

const editLead = (lead: OutreachLead) => {
  editingLeadId.value = lead.id;
  formName.value = lead.person_name;
  formProfileUrl.value = lead.profile_url;
  formHeadline.value = lead.headline || '';
  formRawBio.value = lead.raw_bio;
  formPosts.value = lead.recent_posts && lead.recent_posts.length > 0 
    ? [...lead.recent_posts] 
    : ['', ''];
  formTemplateId.value = lead.template_id || '';
  formCharLimit.value = lead.char_limit || 200;
  formTailoredMessage.value = lead.tailored_message || '';
  formStatus.value = lead.status || 'Draft';
  formCustomInstruction.value = '';
  showForm.value = true;
};

const addPostSlot = () => {
  if (formPosts.value.length < 3) {
    formPosts.value.push('');
  }
};

const removePostSlot = (index: number) => {
  if (formPosts.value.length > 1) {
    formPosts.value.splice(index, 1);
  } else {
    formPosts.value[0] = '';
  }
};

const setCharLimit = (limit: number) => {
  formCharLimit.value = limit;
};

// AI Tailoring
const generateMessage = async () => {
  if (!formName.value.trim()) {
    await dialog.showAlert('Please provide the person\'s name.', 'Validation Error');
    return;
  }
  if (!formRawBio.value.trim()) {
    await dialog.showAlert('Please paste the person\'s profile bio or summary.', 'Validation Error');
    return;
  }

  isGenerating.value = true;
  try {
    const validPosts = formPosts.value.map(p => p.trim()).filter(p => p.length > 0);

    const tailored = await outreachStore.generateTailoredMessage({
      leadId: editingLeadId.value || undefined,
      personName: formName.value.trim(),
      profileUrl: formProfileUrl.value.trim(),
      headline: formHeadline.value.trim() || undefined,
      rawBio: formRawBio.value.trim(),
      recentPosts: validPosts,
      templateId: formTemplateId.value || undefined,
      charLimit: formCharLimit.value || 200,
      customInstruction: formCustomInstruction.value.trim() || undefined,
    });

    formTailoredMessage.value = tailored;
  } catch (err: any) {
    console.error('Error generating tailored outreach:', err);
    await dialog.showAlert(err?.message || 'Failed to tailor outreach message.', 'Generation Error');
  } finally {
    isGenerating.value = false;
  }
};

// Save Lead
const handleSave = async () => {
  if (!formName.value.trim()) {
    await dialog.showAlert('Please enter the person\'s name.', 'Validation Error');
    return;
  }
  if (!formProfileUrl.value.trim()) {
    await dialog.showAlert('Please enter the profile URL.', 'Validation Error');
    return;
  }

  isSaving.value = true;
  try {
    const validPosts = formPosts.value.map(p => p.trim()).filter(p => p.length > 0);

    await outreachStore.saveLead({
      id: editingLeadId.value || undefined,
      personName: formName.value.trim(),
      profileUrl: formProfileUrl.value.trim(),
      headline: formHeadline.value.trim() || undefined,
      rawBio: formRawBio.value.trim(),
      recentPosts: validPosts,
      templateId: formTemplateId.value || undefined,
      charLimit: formCharLimit.value || 200,
      tailoredMessage: formTailoredMessage.value.trim() || undefined,
      status: formStatus.value || 'Draft',
    });

    await dialog.showAlert('Outreach lead saved successfully.', 'Success');
    closeForm();
  } catch (err: any) {
    console.error('Error saving lead:', err);
    await dialog.showAlert('Failed to save outreach lead.', 'Error');
  } finally {
    isSaving.value = false;
  }
};

const handleDeleteLead = async (lead: OutreachLead) => {
  const confirmed = await dialog.showConfirm(
    `Are you sure you want to delete lead for "${lead.person_name}"?`,
    'Delete Lead'
  );

  if (confirmed) {
    try {
      await outreachStore.deleteLead(lead.id);
    } catch (err: any) {
      console.error(err);
      await dialog.showAlert('Failed to delete lead.', 'Error');
    }
  }
};

const copyText = async (text: string, leadId?: string) => {
  if (!text) return;
  try {
    await writeText(text);
    if (leadId) {
      copiedId.value = leadId;
      setTimeout(() => { copiedId.value = null; }, 2000);
    } else {
      formCopied.value = true;
      setTimeout(() => { formCopied.value = false; }, 2000);
    }
  } catch (err) {
    console.error('Failed to copy text:', err);
  }
};

const openProfile = (url: string) => {
  if (!url) return;
  openUrl(url).catch((err: any) => console.error('Failed to open profile URL:', err));
};

const getStatusClass = (status: string) => {
  switch (status) {
    case 'Sent': return 'status-sent';
    case 'Connected': return 'status-connected';
    case 'Replied': return 'status-replied';
    case 'Archived': return 'status-archived';
    default: return 'status-draft';
  }
};

const characterCount = computed(() => formTailoredMessage.value.length);

const characterCountClass = computed(() => {
  const limit = formCharLimit.value;
  const count = characterCount.value;
  if (count === 0) return '';
  if (count <= limit) return 'count-safe';
  return 'count-danger';
});
</script>

<template>
  <div class="outreach-container">
    <!-- Header -->
    <div class="page-header">
      <div class="header-main">
        <div class="header-icon-box">
          <Send :size="24" class="header-icon" />
        </div>
        <div>
          <h2>Direct Outreach</h2>
          <p class="subtitle">
            Hyper-personalized, factually honest executive outreach tailored from profiles and posts with strict character limits.
          </p>
        </div>
      </div>

      <div class="actions">
        <template v-if="!isSelectionMode">
          <!-- Selection Mode Button -->
          <div 
            v-if="outreachStore.leads.length > 0"
            class="btn-tooltip-wrapper" 
            @mouseenter="activeTooltip = 'select-mode'" 
            @mouseleave="activeTooltip = null"
          >
            <button class="btn-action" @click="isSelectionMode = true">
              <CheckSquare :size="16" />
            </button>
            <AnimatePresence>
              <Motion
                v-if="activeTooltip === 'select-mode'"
                :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                :animate="{ opacity: 1, y: 0, scale: 1 }"
                :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                :transition="{ duration: 0.15 }"
                class="floating-message tooltip-top"
              >
                Select Multiple
              </Motion>
            </AnimatePresence>
          </div>

          <!-- Refresh Button -->
          <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'refresh'" @mouseleave="activeTooltip = null">
            <button class="btn-action" @click="outreachStore.loadAllLeads">
              <RotateCw :size="16" :class="{ 'spinner': outreachStore.isLoading }" />
            </button>
            <AnimatePresence>
              <Motion
                v-if="activeTooltip === 'refresh'"
                :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                :animate="{ opacity: 1, y: 0, scale: 1 }"
                :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                :transition="{ duration: 0.15 }"
                class="floating-message tooltip-top"
              >
                Refresh Leads
              </Motion>
            </AnimatePresence>
          </div>

          <!-- New Lead Button -->
          <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'new-lead'" @mouseleave="activeTooltip = null">
            <button class="btn-primary btn-icon-only" @click="openNewForm">
              <Plus :size="16" />
            </button>
            <AnimatePresence>
              <Motion
                v-if="activeTooltip === 'new-lead'"
                :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                :animate="{ opacity: 1, y: 0, scale: 1 }"
                :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                :transition="{ duration: 0.15 }"
                class="floating-message tooltip-top"
              >
                New Lead
              </Motion>
            </AnimatePresence>
          </div>
        </template>

        <!-- Selection Mode Actions -->
        <template v-else>
          <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'select-all'" @mouseleave="activeTooltip = null">
            <button class="btn-action" @click="toggleSelectAll">
              <component :is="selectedIds.size === paginatedLeads.length ? CheckSquare : Square" :size="16" />
            </button>
            <AnimatePresence>
              <Motion
                v-if="activeTooltip === 'select-all'"
                :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                :animate="{ opacity: 1, y: 0, scale: 1 }"
                :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                :transition="{ duration: 0.15 }"
                class="floating-message tooltip-top"
              >
                {{ selectedIds.size === paginatedLeads.length ? 'Deselect All' : 'Select All' }}
              </Motion>
            </AnimatePresence>
          </div>

          <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'delete-batch'" @mouseleave="activeTooltip = null">
            <button
              class="btn-action btn-danger"
              :disabled="selectedIds.size === 0"
              @click="handleBatchDelete"
            >
              <Trash2 :size="16" />
            </button>
            <AnimatePresence>
              <Motion
                v-if="activeTooltip === 'delete-batch'"
                :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                :animate="{ opacity: 1, y: 0, scale: 1 }"
                :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                :transition="{ duration: 0.15 }"
                class="floating-message tooltip-top"
              >
                Delete ({{ selectedIds.size }})
              </Motion>
            </AnimatePresence>
          </div>

          <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'exit-selection'" @mouseleave="activeTooltip = null">
            <button class="btn-action" @click="exitSelectionMode">
              <X :size="16" />
            </button>
            <AnimatePresence>
              <Motion
                v-if="activeTooltip === 'exit-selection'"
                :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                :animate="{ opacity: 1, y: 0, scale: 1 }"
                :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                :transition="{ duration: 0.15 }"
                class="floating-message tooltip-top"
              >
                Done
              </Motion>
            </AnimatePresence>
          </div>
        </template>
      </div>
    </div>

    <!-- Filters & Search Bar -->
    <div class="filter-bar">
      <div class="search-box">
        <Search :size="15" class="search-icon" />
        <input 
          v-model="searchQuery" 
          type="text" 
          placeholder="Search by name, headline, bio..." 
          class="search-input"
        />
        <button v-if="searchQuery" class="clear-search" @click="searchQuery = ''">
          <X :size="13" />
        </button>
      </div>

      <div class="status-filters scroll-tray-x">
        <button 
          v-for="st in ['ALL', 'Draft', 'Sent', 'Connected', 'Replied', 'Archived']" 
          :key="st"
          class="filter-pill"
          :class="{ active: statusFilter === st }"
          @click="statusFilter = st"
        >
          {{ st }}
        </button>
      </div>
    </div>

    <!-- Outreach Lead Creator & Editor Modal Dialog -->
    <AnimatePresence>
      <Motion
        v-if="showForm"
        :initial="{ opacity: 0 }"
        :animate="{ opacity: 1 }"
        :exit="{ opacity: 0 }"
        class="modal-backdrop"
        @click.self="closeForm"
      >
        <Motion
          :initial="{ opacity: 0, scale: 0.95, y: 20 }"
          :animate="{ opacity: 1, scale: 1, y: 0 }"
          :exit="{ opacity: 0, scale: 0.95, y: 20 }"
          :transition="{ duration: 0.2, ease: 'easeOut' }"
          class="form-card"
          @click.stop
        >
          <div class="form-header">
            <div class="form-title">
              <Sparkles :size="18" class="sparkle-icon" />
              <span>{{ editingLeadId ? 'Edit Outreach Lead' : 'Tailor Direct Outreach' }}</span>
            </div>
            <button type="button" class="icon-btn-close" @click="closeForm" title="Close dialog">
              <X :size="18" />
            </button>
          </div>

          <div class="form-modal-body">
            <div class="form-grid">
            <!-- Left Column: Recipient Data & Bio -->
            <div class="form-col">
              <div class="field-group">
                <label class="field-label">Person Name <span class="required">*</span></label>
                <input 
                  v-model="formName" 
                  type="text" 
                  placeholder="e.g. Alex Mercer" 
                  class="text-input" 
                />
              </div>

              <div class="field-group">
                <label class="field-label">Profile URL <span class="required">*</span></label>
                <div class="input-with-action">
                  <input 
                    v-model="formProfileUrl" 
                    type="url" 
                    placeholder="https://linkedin.com/in/alex-mercer" 
                    class="text-input" 
                  />
                  <button 
                    v-if="formProfileUrl" 
                    type="button" 
                    class="field-action-btn"
                    title="Open Link in Browser"
                    @click="openProfile(formProfileUrl)"
                  >
                    <ExternalLink :size="14" />
                  </button>
                </div>
              </div>

              <div class="field-group">
                <label class="field-label">Headline / Title</label>
                <input 
                  v-model="formHeadline" 
                  type="text" 
                  placeholder="e.g. Engineering Lead at Stripe | ex-Google" 
                  class="text-input" 
                />
              </div>

              <div class="field-group">
                <label class="field-label">
                  Profile Bio / Raw Summary <span class="required">*</span>
                  <span class="label-hint">Paste their About section or summary</span>
                </label>
                <textarea 
                  v-model="formRawBio" 
                  rows="4" 
                  placeholder="Paste their LinkedIn 'About', summary, or career background details here..." 
                  class="text-area"
                ></textarea>
              </div>

              <!-- Recent Posts (2-3 posts) -->
              <div class="field-group posts-group">
                <div class="label-row">
                  <label class="field-label">
                    Recent Posts / Insights ({{ formPosts.length }}/3)
                    <span class="label-hint">AI references these authentic topics</span>
                  </label>
                  <button 
                    v-if="formPosts.length < 3" 
                    type="button" 
                    class="btn-add-post" 
                    @click="addPostSlot"
                  >
                    <Plus :size="12" /> Add Post
                  </button>
                </div>

                <div v-for="(_, index) in formPosts" :key="index" class="post-slot">
                  <div class="post-slot-header">
                    <span class="post-slot-tag">Post #{{ index + 1 }}</span>
                    <button 
                      v-if="formPosts.length > 1" 
                      type="button" 
                      class="btn-remove-post" 
                      @click="removePostSlot(index)"
                      title="Remove post"
                    >
                      <X :size="12" />
                    </button>
                  </div>
                  <textarea 
                    v-model="formPosts[index]" 
                    rows="2" 
                    :placeholder="`Paste text from recent post #${index + 1}...`" 
                    class="text-area post-textarea"
                  ></textarea>
                </div>
              </div>
            </div>

            <!-- Right Column: Settings, Generator & Result -->
            <div class="form-col">
              <!-- HR Base Template Dropdown -->
              <div class="field-group">
                <label class="field-label">Base HR Message Template (Optional)</label>
                <CustomSelect
                  v-model="formTemplateId"
                  :options="templateOptions"
                  class="select-input"
                />
              </div>

              <!-- Strict Character Limit Selection -->
              <div class="field-group">
                <div class="label-row">
                  <label class="field-label">
                    Strict Character Limit <span class="required">*</span>
                  </label>
                  <span class="limit-badge">{{ formCharLimit }} chars max</span>
                </div>
                
                <div class="limit-buttons-bar">
                  <button 
                    v-for="preset in PRESETS" 
                    :key="preset"
                    type="button"
                    class="btn-preset"
                    :class="{ active: formCharLimit === preset }"
                    @click="setCharLimit(preset)"
                  >
                    {{ preset }}
                    <span v-if="preset === 200" class="default-badge">Def</span>
                  </button>

                  <div class="custom-limit-box">
                    <span class="custom-limit-label">Custom:</span>
                    <input 
                      v-model.number="formCharLimit" 
                      type="number" 
                      min="50" 
                      max="2000" 
                      step="10" 
                      class="custom-limit-input"
                    />
                  </div>
                </div>
              </div>

              <!-- Custom Instructions -->
              <div class="field-group">
                <label class="field-label">Special Directives (Optional)</label>
                <input 
                  v-model="formCustomInstruction" 
                  type="text" 
                  placeholder="e.g. Mention that I loved their podcast on distributed systems" 
                  class="text-input" 
                />
              </div>

              <!-- AI Tailor Action Button -->
              <div class="tailor-action-bar">
                <button 
                  type="button" 
                  class="btn-tailor-ai" 
                  :disabled="isGenerating || !formName || !formRawBio"
                  @click="generateMessage"
                >
                  <Sparkles :size="16" :class="{ 'spinner': isGenerating }" />
                  <span>{{ isGenerating ? 'Crafting Tailored Outreach...' : 'Tailor Message with AI' }}</span>
                </button>
              </div>

              <!-- Result Message Area -->
              <div class="field-group result-group">
                <div class="label-row">
                  <label class="field-label">Tailored Outreach Message</label>
                  <div class="counter-box" :class="characterCountClass">
                    {{ characterCount }} / {{ formCharLimit }} chars
                  </div>
                </div>

                <textarea 
                  v-model="formTailoredMessage" 
                  rows="6" 
                  placeholder="Generated outreach message will appear here. You can also edit it directly..." 
                  class="text-area result-textarea"
                ></textarea>

                <div class="result-actions">
                  <button
                    type="button"
                    class="btn-result-action"
                    :title="formCopied ? 'Copied!' : 'Copy to Clipboard'"
                    :disabled="!formTailoredMessage"
                    @click="copyText(formTailoredMessage)"
                  >
                    <component :is="formCopied ? Check : Copy" :size="14" />
                  </button>

                  <div class="status-selector-box">
                    <label class="status-label">Status:</label>
                    <CustomSelect
                      v-model="formStatus"
                      :options="statusOptions"
                      class="select-status"
                      placement="top"
                    />
                  </div>
                </div>
              </div>

              <!-- Form Submit Actions -->
              <div class="form-actions">
                <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'form-cancel'" @mouseleave="activeTooltip = null">
                  <button type="button" class="btn-cancel btn-icon-action" @click="closeForm">
                    <X :size="16" />
                  </button>
                  <AnimatePresence>
                    <Motion
                      v-if="activeTooltip === 'form-cancel'"
                      :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                      :animate="{ opacity: 1, y: 0, scale: 1 }"
                      :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                      :transition="{ duration: 0.15 }"
                      class="floating-message tooltip-top"
                    >
                      Cancel
                    </Motion>
                  </AnimatePresence>
                </div>
                <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'form-save'" @mouseleave="activeTooltip = null">
                  <button
                    type="button"
                    class="btn-save btn-icon-action"
                    :disabled="isSaving || !formName || !formProfileUrl"
                    @click="handleSave"
                  >
                    <RotateCw v-if="isSaving" :size="16" class="spinner" />
                    <Save v-else :size="16" />
                  </button>
                  <AnimatePresence>
                    <Motion
                      v-if="activeTooltip === 'form-save'"
                      :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                      :animate="{ opacity: 1, y: 0, scale: 1 }"
                      :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                      :transition="{ duration: 0.15 }"
                      class="floating-message tooltip-top"
                    >
                      {{ isSaving ? 'Saving...' : (editingLeadId ? 'Update Lead' : 'Save Lead') }}
                    </Motion>
                  </AnimatePresence>
                </div>
              </div>
            </div>
          </div>
        </div>
      </Motion>
    </Motion>
  </AnimatePresence>

    <!-- Empty State -->
    <div v-if="filteredLeads.length === 0 && !outreachStore.isLoading" class="empty-state">
      <div class="empty-icon-box">
        <Send :size="32" class="empty-icon" />
      </div>
      <h3>{{ searchQuery || statusFilter !== 'ALL' ? 'No matching leads found' : 'No outreach leads yet' }}</h3>
      <p>
        {{ searchQuery || statusFilter !== 'ALL' 
          ? 'Try adjusting your search query or status filter.' 
          : 'Paste a recruiter or hiring manager\'s profile info and recent posts to craft tailored, character-constrained outreach messages.' 
        }}
      </p>
      <button v-if="!searchQuery && statusFilter === 'ALL'" class="btn-primary empty-btn" @click="openNewForm">
        <Plus :size="16" />
        <span>Create First Outreach Lead</span>
      </button>
    </div>

    <!-- Leads Grid -->
    <div v-else class="leads-grid">
      <div 
        v-for="lead in paginatedLeads" 
        :key="lead.id" 
        class="lead-card"
        :class="{ 'is-selected': selectedIds.has(lead.id) }"
        @click="isSelectionMode ? toggleSelection(lead.id) : null"
      >
        <!-- Selection Checkbox Overlay -->
        <div v-if="isSelectionMode" class="selection-box">
          <component 
            :is="selectedIds.has(lead.id) ? CheckSquare : Square" 
            :size="18" 
            :class="{ 'selected-icon': selectedIds.has(lead.id) }"
          />
        </div>

        <div class="lead-card-header">
          <div class="person-identity">
            <div class="avatar-dot">
              <User :size="14" />
            </div>
            <div class="identity-text">
              <h4 class="person-name">{{ lead.person_name }}</h4>
              <p v-if="lead.headline" class="person-headline">{{ lead.headline }}</p>
            </div>
          </div>

          <div class="header-badges">
            <span class="status-pill" :class="getStatusClass(lead.status)">
              {{ lead.status }}
            </span>
            <button 
              type="button" 
              class="icon-link-btn" 
              title="Open Profile URL" 
              @click.stop="openProfile(lead.profile_url)"
            >
              <ExternalLink :size="13" />
            </button>
          </div>
        </div>

        <!-- Meta info row -->
        <div class="lead-meta-row">
          <span class="meta-tag">
            Limit: {{ lead.char_limit }} chars
          </span>
          <span v-if="lead.recent_posts && lead.recent_posts.length > 0" class="meta-tag">
            {{ lead.recent_posts.length }} post{{ lead.recent_posts.length > 1 ? 's' : '' }} attached
          </span>
          <span v-if="lead.tailored_message" class="meta-tag length-tag">
            Msg: {{ lead.tailored_message.length }} chars
          </span>
        </div>

        <!-- Tailored Message Preview -->
        <div class="message-preview-box">
          <p v-if="lead.tailored_message" class="preview-text">
            {{ lead.tailored_message }}
          </p>
          <p v-else class="no-message-text">
            No tailored message generated yet. Click Edit to tailor.
          </p>
        </div>

        <!-- Card Footer Actions -->
        <div class="lead-card-footer" @click.stop>
          <div class="footer-left">
            <button
              v-if="lead.tailored_message"
              type="button"
              class="card-btn copy-btn"
              :title="copiedId === lead.id ? 'Copied to clipboard' : 'Copy message'"
              @click="copyText(lead.tailored_message, lead.id)"
            >
              <component :is="copiedId === lead.id ? Check : Copy" :size="13" />
            </button>
          </div>

          <div class="footer-right">
            <button type="button" class="card-btn edit-btn" title="Edit / Tailor" @click="editLead(lead)">
              <Edit3 :size="13" />
            </button>
            <button type="button" class="card-btn delete-btn" title="Delete lead" @click="handleDeleteLead(lead)">
              <Trash2 :size="13" />
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Pagination Controls -->
    <div v-if="totalPages > 1" class="pagination-bar">
      <div class="pagination-info">
        Showing <strong>{{ (currentPage - 1) * pageSize + 1 }}</strong> - 
        <strong>{{ Math.min(currentPage * pageSize, filteredLeads.length) }}</strong> of 
        <strong>{{ filteredLeads.length }}</strong> leads
      </div>
      <div class="pagination-controls">
        <button 
          type="button"
          class="btn-pagination" 
          :disabled="currentPage <= 1" 
          @click="currentPage--" 
          title="Previous Page"
        >
          <ChevronLeft :size="16" />
        </button>
        <span class="page-indicator">
          Page <strong>{{ currentPage }}</strong> of <strong>{{ totalPages }}</strong>
        </span>
        <button 
          type="button"
          class="btn-pagination" 
          :disabled="currentPage >= totalPages" 
          @click="currentPage++" 
          title="Next Page"
        >
          <ChevronRight :size="16" />
        </button>
        <div class="page-size-selector">
          <CustomSelect
            v-model="pageSize"
            :options="pageSizeOptions"
            placement="top"
            class="page-size-select"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.outreach-container {
  padding: 40px;
  max-width: 1200px;
  margin: 0 auto;
}

/* Page Header */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 28px;
}

.header-main {
  display: flex;
  align-items: center;
  gap: 16px;
}

.header-icon-box {
  width: 44px;
  height: 44px;
  border-radius: var(--radius-lg, 12px);
  background: var(--surface-soft);
  border: 1px solid var(--line);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--accent);
}

.header-icon {
  color: var(--accent);
}

h2 {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--ink);
  margin: 0 0 4px 0;
  letter-spacing: -0.01em;
}

.subtitle {
  font-size: 0.85rem;
  color: var(--muted);
  margin: 0;
  max-width: 650px;
  line-height: 1.4;
}

.actions {
  display: flex;
  gap: 10px;
  align-items: center;
}

.btn-tooltip-wrapper {
  position: relative;
  display: flex;
}

.btn-primary, .btn-action {
  height: 38px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  border-radius: var(--radius-md, 8px);
  font-size: 0.8rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-primary {
  padding: 0 16px;
  background: var(--accent);
  color: #fff;
  border: none;
}

.btn-primary.btn-icon-only {
  width: 38px;
  min-width: 38px;
  padding: 0;
}

.btn-primary:hover {
  opacity: 0.92;
  transform: translateY(-1px);
}

.btn-action {
  padding: 0 12px;
  background: var(--surface-soft);
  color: var(--ink);
  border: 1px solid var(--line);
}

.btn-action:hover {
  border-color: var(--muted);
  background: var(--surface);
}

.btn-danger {
  color: var(--warning);
  border-color: rgba(248, 81, 73, 0.3);
}

.btn-danger:hover {
  background: rgba(248, 81, 73, 0.1);
  border-color: var(--warning);
}

.btn-danger:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* Tooltip */
.floating-message {
  position: absolute;
  background: var(--accent);
  color: white;
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 0.65rem;
  font-weight: 700;
  white-space: nowrap;
  pointer-events: none;
  z-index: 1000;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.tooltip-top {
  bottom: 120%;
  left: 50%;
  transform: translateX(-50%);
}

.tooltip-top::after {
  content: "";
  position: absolute;
  top: 100%;
  left: 50%;
  transform: translateX(-50%);
  border: 4px solid transparent;
  border-top-color: var(--accent);
}

/* Filter & Search Bar */
.filter-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 24px;
  flex-wrap: wrap;
}

.search-box {
  position: relative;
  display: flex;
  align-items: center;
  flex: 1;
  min-width: 260px;
}

.search-icon {
  position: absolute;
  left: 12px;
  color: var(--muted);
}

.search-input {
  width: 100%;
  height: 40px;
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: var(--radius-md, 8px);
  padding: 0 32px 0 34px;
  font-size: 0.85rem;
  color: var(--ink);
  outline: none;
  transition: border-color 0.15s ease;
}

.search-input:focus {
  border-color: var(--accent);
}

.clear-search {
  position: absolute;
  right: 10px;
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  padding: 2px;
}

.status-filters {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.filter-pill {
  padding: 6px 12px;
  background: var(--surface-soft);
  border: 1px solid var(--line);
  border-radius: 16px;
  font-size: 0.75rem;
  color: var(--muted);
  cursor: pointer;
  transition: all 0.15s ease;
}

.filter-pill:hover {
  color: var(--ink);
  border-color: var(--muted);
}

.filter-pill.active {
  background: var(--accent-soft);
  color: var(--accent);
  border-color: var(--accent);
  font-weight: 600;
}

/* Modal Backdrop & Form Card */
.modal-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.72);
  backdrop-filter: blur(5px);
  -webkit-backdrop-filter: blur(5px);
  z-index: 9990;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  box-sizing: border-box;
}

.form-card {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: var(--radius-lg, 16px);
  width: 100%;
  max-width: 960px;
  max-height: calc(100vh - 48px);
  display: flex;
  flex-direction: column;
  box-shadow: 0 24px 80px rgba(0, 0, 0, 0.6);
  overflow: hidden;
}

.form-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 10px;
  padding: 18px 24px;
  border-bottom: 1px solid var(--line);
  margin-bottom: 0;
  flex-shrink: 0;
}

.form-modal-body {
  padding: 24px;
  overflow-y: auto;
  overflow-x: hidden;
  flex: 1;
  min-height: 0;
  scrollbar-width: thin;
  scrollbar-color: var(--line) transparent;
}

.form-modal-body::-webkit-scrollbar {
  width: 6px;
}

.form-modal-body::-webkit-scrollbar-track {
  background: transparent;
}

.form-modal-body::-webkit-scrollbar-thumb {
  background: var(--line);
  border-radius: 4px;
}

.form-modal-body::-webkit-scrollbar-thumb:hover {
  background: var(--accent);
}

.form-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 1.1rem;
  font-weight: 700;
  color: var(--ink);
  min-width: 0;
  flex: 1;
}

.form-title span {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sparkle-icon {
  color: var(--accent);
  flex-shrink: 0;
}

.icon-btn-close {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  padding: 4px;
  border-radius: 6px;
  transition: all 0.15s;
}

.icon-btn-close:hover {
  color: var(--ink);
  background: var(--surface-soft);
}

.form-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
}

.form-col,
.field-group {
  min-width: 0;
}

/* Tablet (<= 900px): single-column form */
@media (max-width: 900px) {
  .outreach-container {
    padding: 24px 20px;
  }

  .form-grid {
    grid-template-columns: 1fr;
  }
}

/* Mobile (<= 768px, same tier as Jobs/Inbox neighbours): header stays in
   a row with subtitle hidden, filters stack, pills ride the global
   horizontal scroll tray so nothing clips */
@media (max-width: 768px) {
  .outreach-container {
    padding: 16px;
  }

  .page-header {
    flex-direction: row;
    align-items: center;
    gap: 12px;
    margin-bottom: 20px;
  }

  .header-main h2 {
    font-size: 1.3rem;
  }

  .subtitle {
    display: none;
  }

  .actions {
    flex-shrink: 0;
    gap: 8px;
  }

  .btn-primary.btn-icon-only,
  .btn-action {
    width: 36px;
    min-width: 36px;
    height: 36px;
    border-radius: 10px;
    padding: 0;
  }

  .filter-bar {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
    margin-bottom: 16px;
  }

  .search-box {
    min-width: 0;
    width: 100%;
  }

  .search-input {
    padding: 10px 8px;
    font-size: 0.85rem;
  }

  .status-filters {
    width: 100%;
    flex-wrap: nowrap;
    overflow-x: auto;
    overflow-y: hidden;
    -webkit-overflow-scrolling: touch;
    scroll-behavior: smooth;
    padding-bottom: 10px;
    scrollbar-width: thin;
    scrollbar-color: var(--line) transparent;
  }

  .status-filters::-webkit-scrollbar {
    height: 4px;
    display: block;
  }

  .status-filters::-webkit-scrollbar-track {
    background: transparent;
    margin: 0 4px;
  }

  .status-filters::-webkit-scrollbar-thumb {
    background: var(--line);
    border-radius: 4px;
  }

  .status-filters::-webkit-scrollbar-thumb:hover {
    background: var(--accent);
  }

  .filter-pill {
    flex-shrink: 0;
    min-height: 32px;
    display: inline-flex;
    align-items: center;
  }

  .leads-grid {
    grid-template-columns: 1fr;
  }

  .modal-backdrop {
    padding: 12px;
  }

  .form-card {
    max-height: calc(100vh - 24px);
    border-radius: 12px;
    padding: 0;
  }

  .form-header {
    padding: 14px 16px;
  }

  .form-modal-body {
    padding: 16px;
  }

  .form-actions {
    flex-direction: column-reverse;
  }

  .btn-cancel,
  .btn-save {
    width: 100%;
    justify-content: center;
    min-height: 40px;
  }

  .custom-limit-box {
    margin-left: 0;
  }
}

@media (max-width: 480px) {
  .outreach-container {
    padding: 12px 10px;
  }

  .header-icon-box {
    display: none;
  }

  .header-main {
    gap: 10px;
    min-width: 0;
  }

  .person-headline {
    max-width: 150px;
  }

  .modal-backdrop {
    padding: 8px;
  }

  .form-card {
    max-height: calc(100vh - 16px);
  }

  .form-header {
    padding: 12px 14px;
  }

  .form-modal-body {
    padding: 12px;
  }
}

/* Ultra-compact (<= 360px, down to the 300x400 minimum): tighter type,
   scrollable header actions so buttons never clip */
@media (max-width: 360px) {
  .outreach-container {
    padding: 10px 8px;
  }

  .page-header {
    gap: 8px;
    margin-bottom: 14px;
  }

  .header-main {
    min-width: 0;
    flex: 1;
  }

  .header-main h2 {
    font-size: 1.05rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .actions {
    max-width: 100%;
    overflow-x: auto;
    overflow-y: hidden;
    flex-wrap: nowrap;
    padding-bottom: 8px;
    scrollbar-width: none;
  }

  .actions::-webkit-scrollbar {
    display: none;
  }

  .actions > * {
    flex-shrink: 0;
  }

  .filter-bar {
    gap: 8px;
    margin-bottom: 12px;
  }

  .leads-grid {
    gap: 12px;
  }

  .lead-card {
    padding: 12px;
  }

  .person-headline {
    max-width: 110px;
  }

  .modal-backdrop {
    padding: 4px;
  }

  .form-card {
    max-height: calc(100vh - 8px);
    padding: 0;
  }

  .form-header {
    padding: 10px 12px;
  }

  .form-modal-body {
    padding: 10px 8px;
  }

  .form-title {
    font-size: 0.95rem;
  }

  .card-btn {
    min-height: 34px;
  }
}

.field-group {
  margin-bottom: 16px;
}

.label-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 6px;
}

.field-label {
  display: block;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--ink);
  margin-bottom: 6px;
}

.required {
  color: var(--warning);
}

.label-hint {
  font-weight: 400;
  color: var(--muted);
  margin-left: 6px;
  font-size: 0.7rem;
}

.text-input, .select-input, .text-area {
  width: 100%;
  background: var(--surface-soft);
  border: 1px solid var(--line);
  border-radius: var(--radius-md, 8px);
  padding: 11px 12px;
  font-size: 0.85rem;
  color: var(--ink);
  outline: none;
  transition: border-color 0.15s ease;
  font-family: inherit;
}

.text-input:focus, .select-input:focus, .text-area:focus {
  border-color: var(--accent);
}

/* Sleek textarea scrollbars: 4px idle expanding to 6px on hover/focus,
   with track margins so the thumb never touches the edges */
.text-area::-webkit-scrollbar {
  width: 4px;
  transition: all 0.15s ease;
}

.text-area:hover::-webkit-scrollbar,
.text-area:focus::-webkit-scrollbar {
  width: 6px;
}

.text-area::-webkit-scrollbar-track {
  background: transparent;
  margin: 6px 0;
}

.text-area::-webkit-scrollbar-thumb {
  background: var(--line);
  border-radius: 4px;
}

.text-area::-webkit-scrollbar-thumb:hover {
  background: var(--muted);
}

.text-area {
  scrollbar-width: thin;
  scrollbar-color: var(--line) transparent;
}

.input-with-action {
  position: relative;
  display: flex;
  align-items: center;
}

.input-with-action .text-input {
  padding-right: 36px;
}

.field-action-btn {
  position: absolute;
  right: 8px;
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  padding: 4px;
}

.field-action-btn:hover {
  color: var(--accent);
}

.text-area {
  resize: vertical;
  line-height: 1.4;
}

/* Post slots */
.posts-group {
  margin-top: 20px;
}

.btn-add-post {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: var(--surface-soft);
  border: 1px solid var(--line);
  color: var(--accent);
  font-size: 0.7rem;
  font-weight: 600;
  padding: 3px 8px;
  border-radius: 6px;
  cursor: pointer;
}

.btn-add-post:hover {
  border-color: var(--accent);
}

.post-slot {
  background: var(--surface-soft);
  border: 1px solid var(--line);
  border-radius: var(--radius-md, 8px);
  padding: 8px 10px;
  margin-bottom: 10px;
}

.post-slot-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}

.post-slot-tag {
  font-size: 0.65rem;
  font-weight: 700;
  text-transform: uppercase;
  color: var(--muted);
}

.btn-remove-post {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  padding: 2px;
}

.btn-remove-post:hover {
  color: var(--warning);
}

.post-textarea {
  background: var(--surface);
  border: 1px solid var(--line);
}

/* Character Limit Buttons Bar */
.limit-badge {
  font-size: 0.7rem;
  font-weight: 700;
  color: var(--accent);
  background: var(--accent-soft);
  padding: 2px 8px;
  border-radius: 10px;
  white-space: nowrap;
}

.limit-buttons-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.btn-preset {
  position: relative;
  padding: 6px 12px;
  background: var(--surface-soft);
  border: 1px solid var(--line);
  border-radius: 6px;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--ink);
  cursor: pointer;
  transition: all 0.15s ease;
  display: flex;
  align-items: center;
  gap: 4px;
}

.btn-preset:hover {
  border-color: var(--muted);
}

.btn-preset.active {
  background: var(--accent-soft);
  border-color: var(--accent);
  color: var(--accent);
}

.default-badge {
  font-size: 0.6rem;
  background: var(--accent);
  color: white;
  padding: 1px 4px;
  border-radius: 4px;
  text-transform: uppercase;
}

.custom-limit-box {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-left: auto;
}

.custom-limit-label {
  font-size: 0.7rem;
  color: var(--muted);
}

.custom-limit-input {
  width: 84px;
  min-height: 36px;
  background: var(--surface-soft);
  border: 1px solid var(--line);
  border-radius: 6px;
  padding: 8px;
  font-size: 0.8rem;
  color: var(--ink);
  outline: none;
}

/* Native number spinners are OS-rendered and ignore theme vars entirely,
   so hide them: typing + arrow keys + min/max/step still work */
.custom-limit-input::-webkit-outer-spin-button,
.custom-limit-input::-webkit-inner-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.custom-limit-input[type='number'] {
  -moz-appearance: textfield;
  appearance: textfield;
}

.custom-limit-input:focus {
  border-color: var(--accent);
}

/* Tailor AI action */
.tailor-action-bar {
  margin: 16px 0;
}

.btn-tailor-ai {
  width: 100%;
  height: 42px;
  background: var(--accent);
  color: white;
  border: none;
  border-radius: var(--radius-md, 8px);
  font-size: 0.85rem;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-tailor-ai:hover:not(:disabled) {
  opacity: 0.92;
  transform: translateY(-1px);
}

.btn-tailor-ai:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Result Area */
.result-group {
  margin-top: 10px;
}

.counter-box {
  font-size: 0.7rem;
  font-weight: 600;
  color: var(--muted);
  font-family: monospace;
  white-space: nowrap;
}

.count-safe {
  color: var(--accent);
}

.count-danger {
  color: var(--warning);
}

.result-textarea {
  border-color: var(--accent);
  background: var(--surface);
  line-height: 1.5;
  font-size: 0.85rem;
}

.result-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 8px;
  gap: 12px;
  flex-wrap: wrap;
}

.btn-result-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 32px;
  min-height: 32px;
  padding: 6px;
  background: var(--surface-soft);
  border: 1px solid var(--line);
  border-radius: 6px;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--ink);
  cursor: pointer;
  transition: all 0.15s;
}

.btn-result-action:hover:not(:disabled) {
  border-color: var(--accent);
  color: var(--accent);
}

.btn-result-action:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.status-selector-box {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-label {
  font-size: 0.75rem;
  color: var(--muted);
}

.select-status {
  background: var(--surface-soft);
  border: 1px solid var(--line);
  border-radius: 6px;
  padding: 4px 8px;
  font-size: 0.75rem;
  color: var(--ink);
  outline: none;
}

/* CustomSelect root is full-width by default; constrain it inside the
   inline status row so it sizes like the old native select */
.status-selector-box .custom-select-container {
  width: auto;
  min-width: 132px;
}

/* Form Footer Actions */
.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid var(--line);
}

.btn-cancel {
  padding: 8px 16px;
  background: none;
  border: 1px solid var(--line);
  border-radius: var(--radius-md, 8px);
  color: var(--muted);
  font-size: 0.8rem;
  cursor: pointer;
}

.btn-cancel.btn-icon-action,
.btn-save.btn-icon-action {
  width: 38px;
  min-width: 38px;
  height: 38px;
  padding: 0;
  justify-content: center;
}

.btn-cancel:hover {
  color: var(--ink);
  border-color: var(--muted);
}

.btn-save {
  padding: 8px 20px;
  background: var(--accent);
  color: white;
  border: none;
  border-radius: var(--radius-md, 8px);
  font-size: 0.8rem;
  font-weight: 600;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
}

.btn-save:hover:not(:disabled) {
  opacity: 0.92;
}

.btn-save:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Empty State */
.empty-state {
  text-align: center;
  padding: 60px 20px;
  background: var(--surface);
  border: 1px dashed var(--line);
  border-radius: var(--radius-lg, 12px);
  margin-top: 16px;
}

.empty-icon-box {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  background: var(--surface-soft);
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0 auto 16px auto;
  color: var(--muted);
}

.empty-state h3 {
  font-size: 1.1rem;
  color: var(--ink);
  margin: 0 0 8px 0;
}

.empty-state p {
  font-size: 0.85rem;
  color: var(--muted);
  max-width: 480px;
  margin: 0 auto 20px auto;
  line-height: 1.4;
}

.empty-btn {
  margin: 0 auto;
}

/* Leads Grid */
.leads-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(min(340px, 100%), 1fr));
  gap: 16px;
}

.lead-card {
  position: relative;
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: var(--radius-lg, 12px);
  padding: 16px;
  display: flex;
  flex-direction: column;
  transition: all 0.15s ease;
}

.lead-card:hover {
  border-color: var(--muted);
  transform: translateY(-2px);
  box-shadow: 0 6px 18px rgba(0, 0, 0, 0.1);
}

.lead-card.is-selected {
  border-color: var(--accent);
  background: var(--accent-soft);
}

.selection-box {
  position: absolute;
  top: 12px;
  right: 12px;
  color: var(--muted);
  cursor: pointer;
  z-index: 10;
}

.selected-icon {
  color: var(--accent);
}

.lead-card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
  margin-bottom: 10px;
}

.person-identity {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  min-width: 0;
}

.avatar-dot {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: var(--surface-soft);
  border: 1px solid var(--line);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--accent);
  flex-shrink: 0;
}

.identity-text {
  min-width: 0;
}

.person-name {
  font-size: 0.95rem;
  font-weight: 700;
  color: var(--ink);
  margin: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.person-headline {
  font-size: 0.75rem;
  color: var(--muted);
  margin: 2px 0 0 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 220px;
}

.header-badges {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.status-pill {
  font-size: 0.65rem;
  font-weight: 700;
  text-transform: uppercase;
  padding: 2px 8px;
  border-radius: 12px;
  letter-spacing: 0.05em;
}

.status-draft {
  background: var(--surface-soft);
  color: var(--muted);
  border: 1px solid var(--line);
}

.status-sent {
  background: rgba(9, 105, 218, 0.15);
  color: #58a6ff;
  border: 1px solid rgba(88, 166, 255, 0.3);
}

.status-connected {
  background: var(--accent-soft);
  color: var(--accent);
  border: 1px solid var(--accent);
}

.status-replied {
  background: rgba(210, 153, 34, 0.15);
  color: #e3b341;
  border: 1px solid rgba(227, 179, 65, 0.3);
}

.status-archived {
  background: var(--surface-soft);
  color: var(--muted);
  opacity: 0.7;
}

.icon-link-btn {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  padding: 3px;
  border-radius: 4px;
}

.icon-link-btn:hover {
  color: var(--accent);
}

.lead-meta-row {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.meta-tag {
  font-size: 0.65rem;
  color: var(--muted);
  background: var(--surface-soft);
  padding: 2px 6px;
  border-radius: 4px;
  border: 1px solid var(--line);
}

.length-tag {
  font-family: monospace;
  color: var(--accent);
}

.message-preview-box {
  background: var(--bg);
  border: 1px solid var(--line);
  border-radius: var(--radius-md, 8px);
  padding: 10px 12px;
  margin-bottom: 14px;
  flex: 1;
}

.preview-text {
  font-size: 0.78rem;
  line-height: 1.45;
  color: var(--ink);
  margin: 0;
  display: -webkit-box;
  -webkit-line-clamp: 4;
  -webkit-box-orient: vertical;
  overflow: hidden;
  word-break: break-word;
}

.no-message-text {
  font-size: 0.75rem;
  color: var(--muted);
  font-style: italic;
  margin: 0;
}

.lead-card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 10px;
  border-top: 1px solid var(--line);
}

.footer-left, .footer-right {
  display: flex;
  align-items: center;
  gap: 6px;
}

.card-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 30px;
  min-height: 30px;
  padding: 4px 6px;
  background: var(--surface-soft);
  border: 1px solid var(--line);
  border-radius: 6px;
  font-size: 0.72rem;
  font-weight: 500;
  color: var(--ink);
  cursor: pointer;
  transition: all 0.12s;
}

.card-btn:hover {
  border-color: var(--muted);
}

.copy-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
}

.edit-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
}

.delete-btn:hover {
  border-color: var(--warning);
  color: var(--warning);
}

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* Pagination Bar */
.pagination-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 4px 8px 4px;
  margin-top: 16px;
  border-top: 1px solid var(--line);
  flex-wrap: wrap;
  gap: 12px;
}

.pagination-info {
  font-size: 0.85rem;
  color: var(--muted);
}

.pagination-info strong {
  color: var(--ink);
}

.pagination-controls {
  display: flex;
  align-items: center;
  gap: 10px;
}

.btn-pagination {
  background: var(--surface);
  border: 1px solid var(--line);
  color: var(--ink);
  width: 34px;
  height: 34px;
  border-radius: var(--radius-sm, 6px);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-pagination:hover:not(:disabled) {
  border-color: var(--accent);
  color: var(--accent);
}

.btn-pagination:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.page-indicator {
  font-size: 0.85rem;
  color: var(--muted);
  padding: 0 4px;
  white-space: nowrap;
}

.page-indicator strong {
  color: var(--ink);
}

.page-size-select {
  min-width: 120px;
}

@media (max-width: 600px) {
  .pagination-bar {
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 12px;
  }
  .pagination-controls {
    flex-wrap: wrap;
    justify-content: center;
  }
}
</style>
