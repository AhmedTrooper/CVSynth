<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { getName, getVersion, getTauriVersion } from '@tauri-apps/api/app';
import { type } from '@tauri-apps/plugin-os';
import { Code, Copy, Check, ShieldCheck } from '@lucide/vue';
import { openUrl } from '@tauri-apps/plugin-opener';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { Motion } from 'motion-v';

const appName = ref('Roletect');
const appVersion = ref('');
const tauriVersion = ref('');
const osType = ref('');
const identifier = 'com.ahmedtrooper.roletect';
const copied = ref(false);

onMounted(async () => {
  try {
    appName.value = await getName();
    appVersion.value = await getVersion();
    tauriVersion.value = await getTauriVersion();
    osType.value = await type();
  } catch (e) {
    console.error("Failed to load app info:", e);
  }
});

const openLink = (url: string) => {
  openUrl(url).catch((err: any) => console.error('Failed to open URL:', err));
};

const copyIdentifier = async () => {
  await writeText(identifier);
  copied.value = true;
  setTimeout(() => copied.value = false, 2000);
};
</script>

<template>
  <div class="about-container">
    <Motion 
      :initial="{ opacity: 0, scale: 0.95 }"
      :animate="{ opacity: 1, scale: 1 }"
      :transition="{ duration: 0.4, ease: 'easeOut' }"
      class="about-card"
    >
      <div class="card-glow"></div>
      
      <header class="about-header">
        <div class="app-icon-container">
          <div class="app-icon-ring"></div>
          <div class="app-icon-inner">
            <div class="app-icon-core"></div>
          </div>
        </div>
        
        <div class="hero-text">
          <h1 class="app-title">{{ appName }}</h1>
          <div class="badge">
            <ShieldCheck :size="10" />
            <span>Secure Enterprise Build</span>
          </div>
        </div>
      </header>

      <div class="tagline-section">
        <p class="tagline">The precision engine for high-performance LaTeX resume tailoring.</p>
      </div>

      <div class="specs-grid">
        <div 
          class="spec-item clickable" 
          @click="copyIdentifier"
          @keydown.enter="copyIdentifier"
          @keydown.space.prevent="copyIdentifier"
          role="button"
          tabindex="0"
          title="Click to copy identifier"
          aria-label="Click to copy identifier"
        >
          <div class="spec-header">
            <span class="spec-label">IDENTIFIER</span>
            <component :is="copied ? Check : Copy" :size="12" :class="{ 'text-accent': copied }" />
          </div>
          <span class="spec-value mono">{{ identifier }}</span>
        </div>
        <div class="spec-item">
          <div class="spec-header">
            <span class="spec-label">VERSION</span>
          </div>
          <span class="spec-value mono">v{{ appVersion }}</span>
        </div>
        <div class="spec-item">
          <div class="spec-header">
            <span class="spec-label">LICENSE</span>
          </div>
          <span class="spec-value mono">Commercial Proprietary</span>
        </div>
        <div class="spec-item">
          <div class="spec-header">
            <span class="spec-label">PLATFORM</span>
          </div>
          <span class="spec-value mono">{{ osType }}</span>
        </div>
      </div>

      <div class="description-box">
        <p>
          Roletect integrates sovereign LLM orchestration with professional TeX typesetting. 
          Built for professionals who treat their career narrative as a precision specification.
        </p>
      </div>

      <div class="action-row">
        <button 
          class="btn-premium" 
          @click="openLink('https://github.com/AhmedTrooper/RoleTect')"
          title="Open Community, Releases & Documentation in browser"
          aria-label="Open Community, Releases & Documentation in browser"
        >
          <Code :size="16" />
          <span>Community, Releases &amp; Documentation</span>
        </button>
      </div>

      <footer class="about-footer">
        <div class="footer-line"></div>
        <div class="footer-content">
          <p>© 2025-2026 MD. RAMJAN MIAH (AHMEDTROOPER) • ALL RIGHTS RESERVED</p>
        </div>
      </footer>
    </Motion>
  </div>
</template>

<style scoped>
.about-container {
  min-height: 100%;
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-start;
  padding: 32px 20px;
  background: radial-gradient(circle at 50% -20%, rgba(35, 134, 54, 0.05), transparent 70%);
  overflow-y: auto;
  overflow-x: hidden;
  scrollbar-width: thin;
  scrollbar-color: var(--line) transparent;
}

.about-container::-webkit-scrollbar {
  width: 6px;
}

.about-container::-webkit-scrollbar-track {
  background: transparent;
  margin: 6px 0;
}

.about-container::-webkit-scrollbar-thumb {
  background: var(--line);
  border-radius: 4px;
}

.about-container::-webkit-scrollbar-thumb:hover {
  background: var(--accent);
}

.about-card {
  width: 100%;
  max-width: 480px;
  margin: auto 0;
  background: rgba(22, 25, 35, 0.7);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: var(--radius-lg, 24px);
  padding: 36px 32px;
  position: relative;
  box-shadow: 
    0 24px 64px rgba(0, 0, 0, 0.4),
    inset 0 1px 1px rgba(255, 255, 255, 0.05);
  overflow: hidden;
}

.card-glow {
  position: absolute;
  top: 0;
  left: 50%;
  transform: translateX(-50%);
  width: 100%;
  height: 1px;
  background: linear-gradient(90deg, transparent, var(--accent), transparent);
  opacity: 0.5;
}

.about-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  margin-bottom: 24px;
}

.app-icon-container {
  position: relative;
  width: 72px;
  height: 72px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.app-icon-ring {
  position: absolute;
  inset: 0;
  border: 1px solid var(--line);
  border-radius: 22px;
  transform: rotate(45deg);
}

.app-icon-inner {
  width: 50px;
  height: 50px;
  background: var(--bg-accent);
  border: 1px solid var(--line);
  border-radius: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1;
}

.app-icon-core {
  width: 12px;
  height: 12px;
  background: var(--accent);
  border-radius: 50%;
  box-shadow: 0 0 16px var(--accent);
}

.hero-text {
  text-align: center;
}

.app-title {
  font-size: 1.85rem;
  font-weight: 800;
  color: var(--ink);
  margin: 0;
  letter-spacing: -0.03em;
  background: linear-gradient(to bottom, #ffffff, #999999);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: rgba(35, 134, 54, 0.1);
  border: 1px solid rgba(35, 134, 54, 0.25);
  padding: 3px 10px;
  border-radius: 100px;
  margin-top: 6px;
}

.badge span {
  font-size: 0.62rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--accent);
}

.tagline-section {
  text-align: center;
  margin-bottom: 24px;
}

.tagline {
  font-size: 0.85rem;
  color: var(--muted);
  line-height: 1.5;
  margin: 0;
}

.specs-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  margin-bottom: 24px;
}

.spec-item {
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--line);
  border-radius: var(--radius-md, 10px);
  padding: 12px 14px;
  transition: all 0.15s ease;
  cursor: default;
  min-height: 52px;
}

.spec-item.clickable {
  cursor: pointer;
  user-select: none;
}

.spec-item.clickable:hover {
  background: rgba(255, 255, 255, 0.05);
  border-color: var(--accent);
}

.spec-item.clickable:active {
  transform: scale(0.98);
}

.spec-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
  color: var(--muted);
}

.spec-label {
  font-size: 0.62rem;
  font-weight: 700;
  letter-spacing: 0.08em;
}

.spec-value {
  font-size: 0.8rem;
  color: var(--ink);
  display: block;
  word-break: break-all;
}

.mono {
  font-family: 'JetBrains Mono', monospace;
}

.description-box {
  background: var(--bg-accent);
  border-radius: var(--radius-md, 12px);
  padding: 14px 16px;
  margin-bottom: 24px;
  border-left: 3px solid var(--accent);
}

.description-box p {
  font-size: 0.78rem;
  line-height: 1.55;
  color: var(--muted);
  margin: 0;
}

.action-row {
  display: flex;
  width: 100%;
  margin-bottom: 28px;
}

.btn-premium {
  width: 100%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 10px 14px;
  min-height: 42px;
  background: var(--surface-soft);
  border: 1px solid var(--line);
  border-radius: var(--radius-md, 8px);
  color: var(--ink);
  font-size: 0.78rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
  text-align: center;
}

.btn-premium:hover {
  border-color: var(--accent);
  background: var(--surface);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
}

.btn-premium:active {
  transform: scale(0.97);
}

.about-footer {
  text-align: center;
}

.footer-line {
  height: 1px;
  background: linear-gradient(90deg, transparent, var(--line), transparent);
  margin-bottom: 16px;
}

.footer-content {
  display: flex;
  align-items: center;
  justify-content: center;
}

.footer-content p {
  font-size: 0.62rem;
  font-weight: 600;
  color: var(--muted);
  letter-spacing: 0.08em;
  margin: 0;
  line-height: 1.4;
}

.text-accent {
  color: var(--accent);
}

/* =======================================================================
   Tablet Styles (601px - 959px)
   ======================================================================= */
@media (max-width: 959px) and (min-width: 601px) {
  .about-container {
    padding: 24px 16px;
  }

  .about-card {
    padding: 32px 28px;
  }
}

/* =======================================================================
   Mobile Styles (<= 600px):
   1-column specs grid, bounds safety, compact padding for small screens
   ======================================================================= */
@media (max-width: 600px) {
  .about-container {
    padding: 16px 12px;
  }

  .about-card {
    padding: 22px 16px;
    border-radius: var(--radius-md, 16px);
  }

  .app-title {
    font-size: 1.5rem;
  }

  .app-icon-container {
    width: 60px;
    height: 60px;
  }

  .app-icon-ring {
    border-radius: 18px;
  }

  .app-icon-inner {
    width: 42px;
    height: 42px;
    border-radius: 12px;
  }

  .specs-grid {
    grid-template-columns: 1fr;
    gap: 8px;
    margin-bottom: 20px;
  }

  .spec-item {
    padding: 10px 12px;
  }

  .description-box {
    padding: 12px 14px;
    margin-bottom: 20px;
  }

  .action-row {
    margin-bottom: 20px;
  }

  .btn-premium {
    min-height: 42px;
    font-size: 0.75rem;
    padding: 10px 12px;
  }

  .footer-content p {
    font-size: 0.58rem;
    letter-spacing: 0.04em;
  }
}

/* =======================================================================
   Ultra-compact Mobile (<= 340px):
   Fits 300x400 viewports comfortably
   ======================================================================= */
@media (max-width: 340px) {
  .about-container {
    padding: 12px 8px;
  }

  .about-card {
    padding: 16px 10px;
    border-radius: 12px;
  }

  .app-title {
    font-size: 1.3rem;
  }

  .tagline {
    font-size: 0.78rem;
  }

  .btn-premium span {
    font-size: 0.72rem;
  }
}
</style>
