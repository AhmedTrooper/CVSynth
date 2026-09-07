<script setup lang="ts">
import { ref, onMounted, watch, nextTick } from "vue";
import { useRoute } from "vue-router";
import { Motion, AnimatePresence } from "motion-v";
import { openUrl } from "@tauri-apps/plugin-opener";
import Titlebar from "./components/Titlebar.vue";
import SplashLoader from "./components/SplashLoader.vue";
import CustomDialog from "./components/CustomDialog.vue";
import ErrorAuditModal from "./components/ErrorAuditModal.vue";
import CloudUploadOverlay from "./components/CloudUploadOverlay.vue";
import LicenseGate from "./components/LicenseGate.vue";
import { useSettingsStore } from "./store/settings";
import { useLicenseStore } from "./store/license";
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { exit } from '@tauri-apps/plugin-process';
import {
    Home,
    Briefcase,
    Files,
    Settings,
    Code,
    Video,
    Cpu,
    Info,
    Share2,
    Inbox,
    Layers,
    Send,
    ScrollText,
} from "@lucide/vue";

const tabs = [
    { path: "/", label: "Home", icon: Home },
    { path: "/jobs", label: "Jobs", icon: Briefcase },
    { path: "/inbox", label: "Inbox", icon: Inbox },
    { path: "/outreach", label: "Outreach", icon: Send },
    { path: "/templates", label: "Templates", icon: Layers },
    { path: "/documents", label: "Documents", icon: Files },
    { path: "/compiler", label: "Compiler", icon: Cpu },
    { path: "/diagrams", label: "Diagrams", icon: Share2 },
    { path: "/audit", label: "Audit Logs", icon: ScrollText },
    { path: "/settings", label: "Settings", icon: Settings },
    { path: "/about", label: "About", icon: Info },
];

const externalLinks = [
    {
        url: "https://github.com/AhmedTrooper/roletect-app",
        label: "Community & Releases",
        icon: Code,
    },
    {
        url: "https://www.youtube.com/@AhmedTrooper",
        label: "YouTube",
        icon: Video,
    },
];

const route = useRoute();
const navMenuRef = ref<HTMLElement | null>(null);

// Keep active navigation tab centered in horizontal scroll view on mobile/tablet
watch(() => route.path, async () => {
    await nextTick();
    if (navMenuRef.value) {
        const activeEl = navMenuRef.value.querySelector('.nav-item.active') as HTMLElement;
        if (activeEl) {
            activeEl.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'center' });
        }
    }
});

const settingsStore = useSettingsStore();
const licenseStore = useLicenseStore();
const activeTooltip = ref<string | null>(null);
const isAppLoading = ref(true);
const isUploadingToCloud = ref(false);

const handleGateSkip = () => {
    licenseStore.isGateDismissed = true;
    try {
        if (typeof window !== 'undefined' && window.localStorage) {
            localStorage.setItem('license_gate_skipped', 'true');
        }
    } catch {}
    licenseStore.dismissGate();
    settingsStore.enforceFreeTierRestrictions();
};

onMounted(async () => {
    try {
        // Fast synchronous check from local cache
        if (typeof window !== 'undefined' && window.localStorage) {
            if (localStorage.getItem('license_gate_skipped') === 'true') {
                licenseStore.isGateDismissed = true;
            }
        }
        // Load settings, check license validity, and load persisted gate skip status concurrently
        await Promise.allSettled([
            settingsStore.loadSettings(),
            licenseStore.checkLicense(),
            licenseStore.loadGateDismissed()
        ]);
        if (!licenseStore.isLicensed) {
            settingsStore.enforceFreeTierRestrictions();
        }
    } catch (error) {
        console.error("Initialization error:", error);
    } finally {
        isAppLoading.value = false;
    }

    // Randomized background license re-validation while the app stays open, so a
    // deactivated license is caught even if the user never relaunches. Stops once
    // Lemon Squeezy is reached and a definite answer is received.
    licenseStore.startBackgroundRefresh();

    // Intercept window close for cloud backup
    const appWindow = getCurrentWindow();
    appWindow.onCloseRequested(async (event) => {
        // Prevent immediate close
        event.preventDefault();
        
        try {
            // Check if dirty
            const isDirty = await invoke<boolean>('check_data_dirty');
            if (!isDirty) {
                await exit(0);
                return;
            }
            
            // Check if user wants auto local backup
            const autoLocal = await invoke<string>('get_setting', { key: 'auto_local_backup', defaultValue: 'true' });
            if (autoLocal === 'true') {
                try {
                    await invoke('auto_local_backup');
                } catch (e) {
                    console.error("Local backup failed:", e);
                }
            }
            
            // Check if S3 is setup correctly AND auto cloud backup is enabled
            const isSetupOk = await invoke<string>('get_setting', { key: 's3_setup_ok', defaultValue: 'false' });
            const autoCloud = await invoke<string>('get_setting', { key: 'auto_cloud_backup', defaultValue: 'true' });
            if (isSetupOk !== 'true' || autoCloud !== 'true') {
                await exit(0);
                return;
            }
            
            // Show overlay
            isUploadingToCloud.value = true;
            
            // Get credentials from Stronghold
            const ak = (await settingsStore.getSecret('s3_access_key')) || '';
            const sk = (await settingsStore.getSecret('s3_secret_key')) || '';
            
            // Call upload command
            await invoke('upload_backup_to_s3', {
                accessKeyId: ak,
                secretAccessKey: sk
            });
            
            // Brief pause so the user sees it actually completed if it was too fast
            setTimeout(async () => {
                await exit(0);
            }, 600);
            
        } catch (e) {
            console.error("Cloud backup on close failed:", e);
            await exit(0);
        }
    });
});

onMounted(() => {
    document.addEventListener("contextmenu", (e: MouseEvent) => {
        const target = e.target;

        // 1. Ensure target is not null and is an HTML element
        if (target instanceof HTMLElement) {
            // 2. TypeScript now safely recognizes .tagName and .isContentEditable
            if (
                target.tagName === "INPUT" ||
                target.tagName === "TEXTAREA" ||
                target.isContentEditable
            ) {
                return;
            }
        }

        // Block the browser context menu everywhere else
        e.preventDefault();
    });
});

const handleExternalClick = (url: string) => {
    openUrl(url).catch((err: any) => console.error("Failed to open URL:", err));
};
</script>

<template>
    <AnimatePresence>
        <SplashLoader v-if="isAppLoading" key="loader" />
    </AnimatePresence>

    <Titlebar />
    <div class="app-container select-none" @dblclick.prevent>
        <aside class="sidebar">
            <nav class="nav-menu" ref="navMenuRef">
                <router-link
                    v-for="tab in tabs"
                    :key="tab.path"
                    :to="tab.path"
                    class="nav-item"
                    :class="{ active: tab.path === '/templates' ? ($route.path.startsWith('/templates') || $route.path.startsWith('/template')) : undefined }"
                    active-class="active"
                    @mouseenter="activeTooltip = tab.label"
                    @mouseleave="activeTooltip = null"
                >
                    <div class="icon-wrapper">
                        <component :is="tab.icon" :size="20" stroke-width="2" />
                        <AnimatePresence>
                            <Motion
                                v-if="activeTooltip === tab.label"
                                :initial="{ opacity: 0, x: 5, scale: 0.9 }"
                                :animate="{ opacity: 1, x: 12, scale: 1 }"
                                :exit="{ opacity: 0, x: 5, scale: 0.9 }"
                                :transition="{ duration: 0.15 }"
                                class="flying-message sidebar-tooltip"
                            >
                                {{ tab.label }}
                            </Motion>
                        </AnimatePresence>
                    </div>
                </router-link>

                <div class="nav-divider"></div>

                <button
                    v-for="link in externalLinks"
                    :key="link.url"
                    @click="handleExternalClick(link.url)"
                    class="nav-item external"
                    @mouseenter="activeTooltip = link.label"
                    @mouseleave="activeTooltip = null"
                >
                    <div class="icon-wrapper">
                        <component
                            :is="link.icon"
                            :size="20"
                            stroke-width="2"
                        />
                        <AnimatePresence>
                            <Motion
                                v-if="activeTooltip === link.label"
                                :initial="{ opacity: 0, x: 5, scale: 0.9 }"
                                :animate="{ opacity: 1, x: 12, scale: 1 }"
                                :exit="{ opacity: 0, x: 5, scale: 0.9 }"
                                :transition="{ duration: 0.15 }"
                                class="flying-message sidebar-tooltip"
                            >
                                {{ link.label }}
                            </Motion>
                        </AnimatePresence>
                    </div>
                </button>
            </nav>
        </aside>

        <main class="content-area">
            <router-view v-slot="{ Component, route }">
                <transition mode="out-in">
                    <Motion
                        :key="route.path"
                        :initial="{ opacity: 0, y: 5 }"
                        :animate="{ opacity: 1, y: 0 }"
                        :transition="{ duration: 0.15, ease: 'easeOut' }"
                        class="route-wrapper"
                    >
                        <component :is="Component" />
                    </Motion>
                </transition>
            </router-view>
        </main>
    </div>

    <!-- Global Bespoke Dialog System -->
    <CustomDialog />

    <!-- Global Error Audit Trail Modal -->
    <ErrorAuditModal />
    
    <!-- Cloud Upload Overlay -->
    <CloudUploadOverlay :is-visible="isUploadingToCloud" />

    <!-- Strict License Gate Overlay (locks entire app when unlicensed and not dismissed) -->
    <LicenseGate 
      v-if="!licenseStore.isChecking && !licenseStore.isLicensed && !licenseStore.isGateDismissed" 
      @skip="handleGateSkip"
    />
</template>

<style scoped>
/* App Shell Base */
.app-container {
    display: flex;
    flex-direction: column;
    height: calc(100vh - 36px);
    margin-top: 36px;
    width: 100%;
    background: var(--bg);
    color: var(--ink);
    overflow: hidden;
}

.desktop-only {
    display: flex !important;
}

.mobile-only {
    display: none !important;
}

.content-area {
    flex: 1;
    overflow: hidden;
    position: relative;
}

.route-wrapper {
    height: 100%;
    width: 100%;
    overflow-y: auto;
    overflow-x: hidden;
}

/* =======================================================================
   Mobile & Tablet Bottom Bar (<960px):
   Touch-friendly 44px targets, full 20px icons, smooth left-to-right scroll
   ======================================================================= */
.sidebar {
    order: 2;
    background: var(--bg-accent);
    border-top: 1px solid var(--line);
    z-index: 100;
    display: flex;
    align-items: center;
    padding: 0 4px;
    height: 56px;
    flex-shrink: 0;
}

.nav-menu {
    display: flex;
    align-items: center;
    width: 100%;
    height: 100%;
    padding: 3px 6px 7px 6px;
    overflow-x: auto;
    overflow-y: hidden;
    -webkit-overflow-scrolling: touch;
    scroll-behavior: smooth;
    gap: 4px;
    scrollbar-width: thin;
    scrollbar-color: var(--line) transparent;
}

/* Sleek production-grade horizontal scrollbar for mobile/tablet */
.nav-menu::-webkit-scrollbar {
    height: 4px;
    display: block;
}

.nav-menu::-webkit-scrollbar-track {
    background: transparent;
    margin: 0 4px;
}

.nav-menu::-webkit-scrollbar-thumb {
    background: var(--line);
    border-radius: 4px;
}

.nav-menu::-webkit-scrollbar-thumb:hover {
    background: var(--accent);
}

.nav-item {
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 44px;
    height: 38px;
    margin-bottom: 4px; /* Clearance gap above bottom scrollbar */
    padding: 0 10px;
    color: var(--muted);
    text-decoration: none;
    transition: background 0.15s ease, color 0.15s ease, transform 0.1s ease;
    background: none;
    border: none;
    cursor: pointer;
    flex-shrink: 0;
    border-radius: var(--radius-md);
    position: relative;
}

.nav-item:hover {
    color: var(--ink);
    background: var(--surface-soft);
}

.nav-item:active {
    transform: scale(0.94);
}

.nav-item.active {
    color: var(--accent);
    background: var(--accent-soft);
}

.nav-item.active::after {
    content: "";
    position: absolute;
    bottom: 2px;
    left: 50%;
    transform: translateX(-50%);
    width: 16px;
    height: 2px;
    background: var(--accent);
    border-radius: 2px;
}

.nav-divider {
    width: 1px;
    height: 22px;
    background: var(--line);
    margin: 0 4px;
    flex-shrink: 0;
}

.nav-item.external {
    opacity: 0.75;
}

.nav-item.external:hover {
    opacity: 1;
    color: var(--accent);
}

.icon-wrapper {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
}

.icon-wrapper svg {
    width: 20px;
    height: 20px;
}

.flying-message {
    display: none;
}

/* =======================================================================
   Desktop (>= 960px): Vertical Left Sidebar
   ======================================================================= */
@media (min-width: 960px) {
    .app-container {
        flex-direction: row;
    }

    .sidebar {
        order: 0;
        width: 48px;
        height: calc(100vh - 36px);
        flex-direction: column;
        border-top: none;
        border-right: 1px solid var(--line);
        padding: 12px 0;
        align-items: center;
    }

    .nav-menu {
        flex-direction: column;
        gap: 8px;
        padding: 0;
        overflow-x: visible;
        overflow-y: auto;
        scrollbar-width: none;
    }

    .nav-menu::-webkit-scrollbar {
        display: none;
    }

    .nav-divider {
        width: 24px;
        height: 1px;
        margin: 8px 0;
    }

    .nav-item {
        width: 36px;
        min-width: 36px;
        height: 36px;
        padding: 0;
        margin-bottom: 0;
    }

    .nav-item.active::after {
        display: none;
    }

    .nav-item.active::before {
        content: "";
        position: absolute;
        left: -6px;
        top: 6px;
        bottom: 6px;
        width: 2px;
        background: var(--accent);
        border-radius: 2px;
    }

    .flying-message {
        display: block;
        position: absolute;
        left: 100%;
        top: 50%;
        transform: translateY(-50%);
        margin-left: 12px;
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

    .flying-message::after {
        content: "";
        position: absolute;
        top: 50%;
        right: 100%;
        transform: translateY(-50%);
        border: 4px solid transparent;
        border-right-color: var(--accent);
    }
}
</style>
