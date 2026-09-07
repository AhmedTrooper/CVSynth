<script setup lang="ts">
import { useDialogStore } from '../store/dialog';
import { Motion, AnimatePresence } from 'motion-v';
import { X, Info, HelpCircle, FileInput, Calendar, AlertTriangle, Copy, Check } from '@lucide/vue';
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { VueDatePicker } from '@vuepic/vue-datepicker';
import '@vuepic/vue-datepicker/dist/main.css';
import { copyToClipboard } from '../utils/clipboard';

const store = useDialogStore();
const inputRef = ref<HTMLInputElement | null>(null);
const isCopied = ref(false);
const activeTooltip = ref<string | null>(null);

const isErrorMessage = computed(() => {
  const title = (store.options?.title || '').toLowerCase();
  const msg = (store.options?.message || '').toLowerCase();
  return (
    title.includes('error') ||
    title.includes('failed') ||
    title.includes('fail') ||
    title.includes('invalid') ||
    title.includes('exception') ||
    title.includes('warning') ||
    msg.includes('error:') ||
    msg.includes('failed:')
  );
});

const handleCopyMessage = async () => {
  if (!store.options?.message) return;
  const title = store.options.title;
  const msg = store.options.message;
  let textToCopy = msg;
  if (
    title &&
    !['System Message', 'Notification', 'Alert', 'Confirm Action'].includes(title) &&
    !msg.toLowerCase().includes(title.toLowerCase())
  ) {
    textToCopy = `${title}: ${msg}`;
  }
  const ok = await copyToClipboard(textToCopy);
  if (ok) {
    isCopied.value = true;
    setTimeout(() => {
      isCopied.value = false;
    }, 2000);
  }
};

const handleConfirm = () => {
  if (store.options?.type === 'prompt' || store.options?.type === 'datepicker') {
    store.options.onConfirm(store.inputValue);
  } else {
    store.options?.onConfirm();
  }
};

const handleCancel = () => {
  store.options?.onCancel();
};

const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') handleCancel();
  if (e.key === 'Enter' && store.options?.type !== 'prompt' && store.options?.type !== 'datepicker') handleConfirm();
};

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});
</script>

<template>
  <AnimatePresence>
    <div v-if="store.isOpen" class="dialog-overlay">
      <Motion
        :initial="{ opacity: 0 }"
        :animate="{ opacity: 1 }"
        :exit="{ opacity: 0 }"
        class="backdrop"
        @click="handleCancel"
      />
      
      <Motion
        :initial="{ opacity: 0, scale: 0.9, y: 20 }"
        :animate="{ opacity: 1, scale: 1, y: 0 }"
        :exit="{ opacity: 0, scale: 0.9, y: 20 }"
        :transition="{ type: 'spring', damping: 25, stiffness: 300 }"
        class="dialog-card"
        :class="{ 'datepicker-mode': store.options?.type === 'datepicker' }"
      >
        <div class="dialog-header">
          <div class="header-left">
            <AlertTriangle v-if="store.options?.type === 'alert' && isErrorMessage" :size="18" class="icon error-icon" />
            <Info v-else-if="store.options?.type === 'alert'" :size="18" class="icon alert-icon" />
            <HelpCircle v-else-if="store.options?.type === 'confirm'" :size="18" class="icon confirm-icon" />
            <FileInput v-else-if="store.options?.type === 'prompt'" :size="18" class="icon prompt-icon" />
            <Calendar v-else :size="18" class="icon datepicker-icon" />
            <span class="dialog-title">{{ store.options?.title || 'System Message' }}</span>
          </div>
          <div class="header-right">
            <button
              v-if="store.options?.message"
              type="button"
              class="header-copy-btn"
              @click="handleCopyMessage"
              :title="isCopied ? 'Copied!' : (isErrorMessage ? 'Copy Error' : 'Copy Message')"
            >
              <Check v-if="isCopied" :size="14" class="copied-icon" />
              <Copy v-else :size="14" />
            </button>
            <button class="close-btn" @click="handleCancel" title="Close">
              <X :size="16" />
            </button>
          </div>
        </div>

        <div class="dialog-body">
          <p class="dialog-message">{{ store.options?.message }}</p>
          
          <div v-if="store.options?.type === 'prompt'" class="input-wrapper">
            <input 
              ref="inputRef"
              v-model="store.inputValue" 
              class="dialog-input" 
              :placeholder="store.options.defaultValue"
              @keyup.enter="handleConfirm"
              autofocus
            />
          </div>

          <div v-else-if="store.options?.type === 'datepicker'" class="input-wrapper datepicker-wrapper">
            <VueDatePicker 
              v-model="store.inputValue"
              dark
              inline
              auto-apply
              :enable-time-picker="false"
              format="yyyy-MM-dd"
              model-type="yyyy-MM-dd"
              class="custom-datepicker"
            />
          </div>
        </div>

        <div class="dialog-footer">
          <div class="footer-left">
            <div 
              v-if="store.options?.message" 
              class="btn-tooltip-wrapper" 
              @mouseenter="activeTooltip = 'copy'" 
              @mouseleave="activeTooltip = null"
            >
              <button
                type="button"
                class="btn-icon-dialog btn-copy-dialog"
                @click="handleCopyMessage"
                :title="isCopied ? 'Copied!' : (isErrorMessage ? 'Copy Error' : 'Copy Message')"
              >
                <Check v-if="isCopied" :size="16" class="copied-icon" />
                <Copy v-else :size="16" />
              </button>
              <AnimatePresence>
                <Motion
                  v-if="activeTooltip === 'copy'"
                  :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                  :animate="{ opacity: 1, y: 0, scale: 1 }"
                  :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                  :transition="{ duration: 0.15 }"
                  class="flying-message tooltip-top"
                >
                  {{ isCopied ? 'Copied!' : (isErrorMessage ? 'Copy Error' : 'Copy Message') }}
                </Motion>
              </AnimatePresence>
            </div>
          </div>
          <div class="footer-right">
            <div 
              v-if="store.options?.type !== 'alert'" 
              class="btn-tooltip-wrapper" 
              @mouseenter="activeTooltip = 'cancel'" 
              @mouseleave="activeTooltip = null"
            >
              <button 
                class="btn-icon-dialog btn-cancel" 
                @click="handleCancel"
                :title="store.options?.cancelText || 'Cancel'"
              >
                <X :size="16" />
              </button>
              <AnimatePresence>
                <Motion
                  v-if="activeTooltip === 'cancel'"
                  :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                  :animate="{ opacity: 1, y: 0, scale: 1 }"
                  :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                  :transition="{ duration: 0.15 }"
                  class="flying-message tooltip-top"
                >
                  {{ store.options?.cancelText || 'Cancel' }}
                </Motion>
              </AnimatePresence>
            </div>

            <div 
              class="btn-tooltip-wrapper" 
              @mouseenter="activeTooltip = 'confirm'" 
              @mouseleave="activeTooltip = null"
            >
              <button 
                class="btn-icon-dialog btn-confirm" 
                @click="handleConfirm"
                :title="store.options?.confirmText || (store.options?.type === 'alert' ? 'Got it' : 'Confirm')"
              >
                <Check :size="16" />
              </button>
              <AnimatePresence>
                <Motion
                  v-if="activeTooltip === 'confirm'"
                  :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                  :animate="{ opacity: 1, y: 0, scale: 1 }"
                  :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                  :transition="{ duration: 0.15 }"
                  class="flying-message tooltip-top"
                >
                  {{ store.options?.confirmText || (store.options?.type === 'alert' ? 'Got it' : 'Confirm') }}
                </Motion>
              </AnimatePresence>
            </div>
          </div>
        </div>
      </Motion>
    </div>
  </AnimatePresence>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  z-index: 100000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  overflow: hidden;
}

.backdrop {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(4px);
}

.dialog-card {
  position: relative;
  width: 100%;
  max-width: 500px;
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 16px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.6);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.dialog-card.datepicker-mode {
  max-width: 380px;
}

.dialog-header {
  height: 48px;
  padding: 0 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-accent);
  border-bottom: 1px solid var(--line);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.dialog-title {
  font-size: 0.8rem;
  font-weight: 800;
  color: var(--ink);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.icon {
  color: var(--accent);
}

.alert-icon { color: var(--accent); }
.error-icon { color: var(--warning); }
.confirm-icon { color: #4cc9f0; }
.prompt-icon { color: #a371f7; }
.datepicker-icon { color: var(--accent); }
.copied-icon { color: var(--accent); }

.header-right {
  display: flex;
  align-items: center;
  gap: 6px;
}

.header-copy-btn {
  background: var(--surface-soft);
  border: 1px solid var(--line);
  color: var(--muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px 6px;
  border-radius: 6px;
  transition: all 0.2s;
}

.header-copy-btn:hover {
  background: var(--surface);
  color: var(--accent);
  border-color: var(--accent);
}

.close-btn {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  display: flex;
  padding: 4px;
  border-radius: 6px;
}

.close-btn:hover {
  background: var(--surface-soft);
  color: var(--ink);
}

.dialog-body {
  padding: 24px;
}

.dialog-message {
  margin: 0;
  font-size: 0.95rem;
  line-height: 1.6;
  color: var(--ink);
  margin-bottom: 16px;
  white-space: pre-wrap;
  user-select: text !important;
  -webkit-user-select: text !important;
  cursor: text;
  max-height: 360px;
  overflow-y: auto;
  word-break: break-word;
}

/* Sleek 2-to-5 unit scrollbar for dialog message with track margin */
.dialog-message::-webkit-scrollbar {
  width: 4px;
  height: 2px;
  transition: all 0.15s ease;
}

.dialog-message:hover::-webkit-scrollbar,
.dialog-message:focus-within::-webkit-scrollbar {
  width: 6px;
  height: 5px;
}

.dialog-message::-webkit-scrollbar-track {
  background: transparent;
  margin: 6px 0;
}

.dialog-message::-webkit-scrollbar-thumb {
  background: var(--line);
  border-radius: 4px;
}

.dialog-message::-webkit-scrollbar-thumb:hover {
  background: var(--muted);
}

.input-wrapper {
  margin-top: 8px;
}

.dialog-input {
  width: 100%;
  padding: 12px 16px;
  background: var(--bg);
  border: 1px solid var(--line);
  border-radius: 8px;
  color: var(--ink);
  font-size: 1rem;
  outline: none;
}

.dialog-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-soft);
}

.dialog-footer {
  padding: 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  background: var(--bg-accent);
  border-top: 1px solid var(--line);
}

.footer-left {
  display: flex;
  align-items: center;
}

.footer-right {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-left: auto;
}

.btn-tooltip-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.flying-message {
  position: absolute;
  background: var(--surface-soft);
  color: var(--ink);
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 0.65rem;
  font-weight: 700;
  white-space: nowrap;
  pointer-events: none;
  z-index: 1000;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  border: 1px solid var(--line);
}

.tooltip-top {
  bottom: calc(100% + 8px);
  left: 50%;
  transform: translateX(-50%);
}

.btn-icon-dialog {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  padding: 0;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-copy-dialog {
  background: var(--surface-soft);
  border: 1px solid var(--line);
  color: var(--ink);
}

.btn-copy-dialog:hover {
  background: var(--surface);
  border-color: var(--accent);
  color: var(--accent);
}

.btn-confirm {
  background: var(--accent);
  color: white;
  border: none;
}

.btn-confirm:hover {
  filter: brightness(1.1);
  box-shadow: 0 0 12px var(--accent-soft);
}

.btn-cancel {
  background: var(--surface-soft);
  border: 1px solid var(--line);
  color: var(--ink);
}

.btn-cancel:hover {
  background: var(--surface);
  border-color: var(--muted);
  color: var(--warning);
}

/* Datepicker Theming Overrides */
:deep(.dp__main) {
  font-family: inherit;
}

:deep(.dp__theme_dark) {
  --dp-background-color: var(--bg);
  --dp-text-color: var(--ink);
  --dp-hover-color: var(--surface-soft);
  --dp-hover-text-color: var(--ink);
  --dp-hover-icon-color: var(--accent);
  --dp-primary-color: var(--accent);
  --dp-primary-disabled-color: var(--muted);
  --dp-primary-text-color: #ffffff;
  --dp-secondary-color: var(--muted);
  --dp-border-color: var(--line);
  --dp-menu-border-color: var(--line);
  --dp-border-color-hover: var(--accent);
  --dp-disabled-color: var(--surface-soft);
  --dp-scroll-bar-background: var(--bg);
  --dp-scroll-bar-color: var(--muted);
  --dp-success-color: var(--accent);
  --dp-success-color-disabled: var(--muted);
  --dp-icon-color: var(--muted);
  --dp-danger-color: var(--warning);
  --dp-highlight-color: var(--accent-soft);
}

:deep(.dp__outer_menu_wrap) {
  width: 100%;
}

:deep(.dp__menu) {
  border: none !important;
  background: transparent !important;
}

:deep(.dp__calendar_header_item) {
  font-weight: 700;
  font-size: 0.7rem;
  color: var(--muted);
}

:deep(.dp__cell_inner) {
  border-radius: 8px;
  font-size: 0.85rem;
}

:deep(.dp__active_date) {
  background: var(--accent) !important;
}

:deep(.dp__today) {
  border: 1px solid var(--accent) !important;
}
</style>