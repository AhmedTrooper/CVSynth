<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { Motion, AnimatePresence } from 'motion-v';
import { AlertTriangle, X } from '@lucide/vue';
import { useErrorAuditStore } from '../store/error_audit';
import ErrorAuditViewer from './ErrorAuditViewer.vue';

const errorStore = useErrorAuditStore();

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Escape' && errorStore.isViewerOpen) {
    errorStore.closeViewer();
  }
};

onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
});
</script>

<template>
  <AnimatePresence>
    <div
      v-if="errorStore.isViewerOpen"
      key="error-audit-modal"
      class="audit-modal-backdrop"
      @click.self="errorStore.closeViewer()"
    >
      <Motion
        :initial="{ opacity: 0, scale: 0.95, y: 15 }"
        :animate="{ opacity: 1, scale: 1, y: 0 }"
        :exit="{ opacity: 0, scale: 0.95, y: 15 }"
        :transition="{ duration: 0.2, ease: 'easeOut' }"
        class="audit-modal-container"
      >
        <!-- Modal Header -->
        <div class="modal-header">
          <div class="header-title-group">
            <div class="icon-circle">
              <AlertTriangle :size="18" class="header-icon" />
            </div>
            <div>
              <h3 class="modal-title">System Error Audit Trail</h3>
              <p class="modal-subtitle">
                Time-by-time error ledger capturing creating, compiling, fetching, and AI tasks.
              </p>
            </div>
          </div>

          <button
            type="button"
            class="close-modal-btn"
            title="Close (Esc)"
            @click="errorStore.closeViewer()"
          >
            <X :size="18" />
          </button>
        </div>

        <!-- Modal Body with ErrorAuditViewer -->
        <div class="modal-body">
          <ErrorAuditViewer />
        </div>
      </Motion>
    </div>
  </AnimatePresence>
</template>

<style scoped>
.audit-modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 100001;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.audit-modal-container {
  width: 100%;
  max-width: 960px;
  max-height: 88vh;
  display: flex;
  flex-direction: column;
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 12px;
  box-shadow: 0 24px 48px rgba(0, 0, 0, 0.6), 0 0 1px rgba(255, 255, 255, 0.1);
  overflow: hidden;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--line);
  background: var(--bg-accent);
  flex-shrink: 0;
}

.header-title-group {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.icon-circle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 8px;
  background: rgba(248, 81, 73, 0.12);
  border: 1px solid var(--warning);
  flex-shrink: 0;
}

.header-icon {
  color: var(--warning);
}

.modal-title {
  margin: 0;
  font-size: 1.05rem;
  font-weight: 600;
  color: var(--ink);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.modal-subtitle {
  margin: 2px 0 0 0;
  font-size: 0.78rem;
  color: var(--muted);
}

.close-modal-btn {
  background: none;
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  color: var(--muted);
  cursor: pointer;
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.close-modal-btn:hover {
  background: var(--surface-soft);
  border-color: var(--line);
  color: var(--ink);
}

.close-modal-btn:active {
  background: var(--surface);
  transform: scale(0.92);
}

.modal-body {
  padding: 16px 20px 20px 20px;
  overflow-y: auto;
  flex: 1;
}

/* =======================================================================
   Tablet Styles (601px - 959px)
   ======================================================================= */
@media (max-width: 959px) and (min-width: 601px) {
  .audit-modal-backdrop {
    padding: 16px;
  }

  .audit-modal-container {
    max-width: 92vw;
    max-height: 90vh;
  }

  .modal-header {
    padding: 14px 16px;
  }

  .modal-body {
    padding: 14px 16px 16px 16px;
  }
}

/* =======================================================================
   Mobile Styles (<= 600px):
   Touch-friendly layout, maximized screen area, bounds protection
   ======================================================================= */
@media (max-width: 600px) {
  .audit-modal-backdrop {
    padding: 8px;
  }

  .audit-modal-container {
    max-height: 94vh;
    border-radius: var(--radius-lg);
  }

  .modal-header {
    padding: 10px 12px;
  }

  .header-title-group {
    gap: 8px;
  }

  .icon-circle {
    width: 32px;
    height: 32px;
    border-radius: 6px;
  }

  .modal-title {
    font-size: 0.92rem;
    font-weight: 700;
  }

  .modal-subtitle {
    display: none;
  }

  .modal-body {
    padding: 10px 10px 12px 10px;
  }
}

/* =======================================================================
   Ultra-compact Mobile (<= 340px width):
   Ensure clean presentation down to 300px width
   ======================================================================= */
@media (max-width: 340px) {
  .audit-modal-backdrop {
    padding: 4px;
  }

  .modal-header {
    padding: 8px 10px;
  }

  .icon-circle {
    width: 28px;
    height: 28px;
  }

  .modal-title {
    font-size: 0.85rem;
  }

  .modal-body {
    padding: 8px 6px 10px 6px;
  }
}

/* =======================================================================
   Compact Viewport Height (<= 450px height):
   Tighten header and padding for 400px height viewports
   ======================================================================= */
@media (max-height: 450px) {
  .audit-modal-backdrop {
    padding: 4px;
  }

  .audit-modal-container {
    max-height: 96vh;
  }

  .modal-header {
    padding: 6px 10px;
  }

  .icon-circle {
    width: 26px;
    height: 26px;
  }

  .modal-title {
    font-size: 0.85rem;
  }

  .modal-subtitle {
    display: none;
  }

  .modal-body {
    padding: 6px 8px 8px 8px;
  }
}
</style>
