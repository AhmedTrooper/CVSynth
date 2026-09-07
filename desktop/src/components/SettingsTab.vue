<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue';
import { useSettingsStore } from '../store/settings';
import { useLicenseStore } from '../store/license';
import { 
  CheckCircle, 
  Info, 
  Save, 
  RotateCcw, 
  Download, 
  Database, 
  Upload, 
  RefreshCw,
  Palette,
  Plus,
  Trash2,
  Type,
  Italic,
  Play,
  DownloadCloud,
  ShieldCheck,
  LogOut,
  ExternalLink,
  Sparkles,
  ClipboardPaste,
  Lock,
  Copy,
  Check,
  AlertTriangle,
  FolderOpen,
  FileText,
  X
} from '@lucide/vue';
import { copyToClipboard } from '../utils/clipboard';
import { Motion, AnimatePresence } from 'motion-v';
import { invoke } from '@tauri-apps/api/core';
import { save as saveDialog, open as openDialog } from '@tauri-apps/plugin-dialog';
import { writeTextFile, readTextFile } from '@tauri-apps/plugin-fs';
import { openUrl } from '@tauri-apps/plugin-opener';

import { readText, writeText } from '@tauri-apps/plugin-clipboard-manager';
import { useDialogStore } from '../store/dialog';
import { ask } from '@tauri-apps/plugin-dialog';
import { useRoute } from 'vue-router';
import CustomSelect from './CustomSelect.vue';
import ErrorAuditViewer from './ErrorAuditViewer.vue';

const store = useSettingsStore();
const licenseStore = useLicenseStore();
const route = useRoute();
const dialog = useDialogStore();
const isDeactivatingLicense = ref(false);
const upgradeKeyInput = ref('');
const isActivatingUpgrade = ref(false);
const isSyncingLicenseSettings = ref(false);
const copiedInlineError = ref<string | null>(null);

const handleCopyInlineError = async (text: string) => {
  if (!text) return;
  const ok = await copyToClipboard(text);
  if (ok) {
    copiedInlineError.value = text;
    setTimeout(() => {
      if (copiedInlineError.value === text) {
        copiedInlineError.value = null;
      }
    }, 2000);
  }
};

const handleSyncLicenseSettings = async () => {
  isSyncingLicenseSettings.value = true;
  try {
    const success = await licenseStore.refreshLicense();
    if (success) {
      const isTrial = licenseStore.licenseStatus?.trial;
      const status = licenseStore.licenseStatus?.status;
      if (!isTrial && status === 'active') {
        await dialog.showAlert('License verified successfully! Your subscription is active as Pro Member.', 'License Verified');
      } else if (isTrial) {
        await dialog.showAlert(`License verified! Trial is active (${licenseStore.licenseStatus?.trial_ends_at ? new Date(licenseStore.licenseStatus.trial_ends_at).toLocaleDateString() : 'Active'}).`, 'Trial Active');
      } else {
        await dialog.showAlert(`License status: ${status}.`, 'License Status');
      }
    } else if (licenseStore.isLicensed) {
      await dialog.showAlert(
        "Couldn't reach Lemon Squeezy right now. Using your saved license for the grace period \u2014 your access stays active and will re-check automatically.",
        'Offline Grace'
      );
    } else {
      await dialog.showAlert('Your license is no longer active (deactivated or expired). RoleTect is now locked. Activate a valid license key to continue.', 'License Inactive');
    }
  } catch (err: any) {
    await dialog.showAlert(err.toString(), 'Verification Error');
  } finally {
    isSyncingLicenseSettings.value = false;
  }
};

// Explicit paste button (desktop apps don't always expose a native context menu).
const handlePasteUpgradeKey = async () => {
  try {
    const text = await readText();
    if (text) upgradeKeyInput.value = text;
  } catch (e) {
    console.error('Failed to read clipboard:', e);
  }
};

const handleActivateUpgrade = async () => {
  if (!upgradeKeyInput.value.trim()) return;
  isActivatingUpgrade.value = true;
  try {
    const success = await licenseStore.activateLicense(upgradeKeyInput.value);
    if (success) {
      await dialog.showAlert('License successfully activated! Your copy of RoleTect is now fully unlocked.', 'License Activated');
      upgradeKeyInput.value = '';
    } else {
      await dialog.showAlert(licenseStore.activationError || 'Activation failed. Please check your key.', 'Activation Error');
    }
  } catch (err: any) {
    await dialog.showAlert(err.toString(), 'Activation Error');
  } finally {
    isActivatingUpgrade.value = false;
  }
};

// Opens Lemon Squeezy's customer orders page, where the user cancels their own
// subscription or trial. No store API key needed; billing stays with Lemon Squeezy.
const handleCancelSubscription = async () => {
  await openUrl('https://app.lemonsqueezy.com/my-orders/').catch((e: any) =>
    console.error('Failed to open subscription management page:', e)
  );
};

const handleDeactivateLicense = async () => {
  const confirmed = await ask('Are you sure you want to deactivate your license on this device? This will release the seat on Lemon Squeezy and lock RoleTect until a valid key is entered.', {
    title: 'Deactivate License',
    kind: 'warning'
  });
  if (!confirmed) return;
  isDeactivatingLicense.value = true;
  try {
    // Verifies server deactivation with Lemon Squeezy API
    await licenseStore.deactivateLicense();
    // Instantly mounts LicenseGate overlay across the entire viewport
  } catch (err: any) {
    await dialog.showAlert(err.message || err.toString() || 'Failed to deactivate license with Lemon Squeezy. Please check your internet connection.', 'Deactivation Failed');
  } finally {
    isDeactivatingLicense.value = false;
  }
};

// Tooltip State
const activeTooltip = ref<string | null>(null);

// Theme State
const isImportingTheme = ref(false);
const customThemeJson = ref('');
const themeError = ref('');

// Typography Data
const fontFamilies = [
  { id: 'Inter', name: 'Inter (Sans)' },
  { id: 'Geist Sans', name: 'Geist (Modern)' },
  { id: 'Merriweather', name: 'Merriweather (Serif)' },
  { id: 'JetBrains Mono', name: 'JetBrains Mono (Code)' }
];

const fontWeights = [
  { id: '300', name: 'Light' },
  { id: '400', name: 'Regular' },
  { id: '500', name: 'Medium' },
  { id: '600', name: 'Semi-Bold' },
  { id: '700', name: 'Bold' }
];

const fontStyles = [
  { id: 'normal', name: 'Normal' },
  { id: 'italic', name: 'Italic' }
];

const copyDemoTheme = async () => {
  const demoTheme = {
    name: "Surgical Neon",
    colors: {
      "--bg": "#0a0a0a",
      "--bg-accent": "#121212",
      "--surface": "#1a1a1a",
      "--surface-soft": "#242424",
      "--ink": "#ffffff",
      "--muted": "#666666",
      "--line": "#333333",
      "--accent": "#00ff9d",
      "--accent-soft": "rgba(0, 255, 157, 0.1)",
      "--warning": "#ff3e3e"
    }
  };
  
  try {
    await writeText(JSON.stringify(demoTheme, null, 2));
    await dialog.showAlert('Demo theme JSON copied to clipboard!', 'Success');
  } catch (err) {
    console.error('Failed to copy to clipboard:', err);
  }
};

// --- 1. Draft State (Local only) ---
const providerInput = ref('');
const modelInput = ref('');
const apiKeyInput = ref('');
const customBaseUrlInput = ref('');
const customModelInput = ref('');
const savedCustomBaseUrl = ref('');
const savedCustomModel = ref('');

// UI feedback states
const isSaving = ref(false);
const isExporting = ref(false);
const isImporting = ref(false);
const isClearingCache = ref(false);
const showSuccess = ref(false);
const saveError = ref('');

const handleClearCache = async () => {
  const confirmed = await dialog.showConfirm(
    'This will delete the entire Tectonic cache. It will be rebuilt automatically during the next compilation, which may take some time. Proceed?',
    'Purge LaTeX Cache'
  );
  if (!confirmed) return;

  isClearingCache.value = true;
  try {
    await invoke('clear_tectonic_cache');
    await dialog.showAlert('Tectonic cache has been successfully purged.', 'Cache Cleared');
  } catch (err: any) {
    await dialog.showAlert(`Failed to clear cache: ${err.toString()}`, 'Error');
  } finally {
    isClearingCache.value = false;
  }
};

const exportData = async () => {
  isExporting.value = true;
  try {
    const data = await invoke('export_all_data');
    const now = new Date();
    const timestamp = now.toISOString().replace(/[:.]/g, '-').split('T');
    const dateStr = timestamp[0];
    const timeStr = timestamp[1].split('Z')[0];
    
    const path = await saveDialog({
      filters: [{ name: 'JSON', extensions: ['json'] }],
      defaultPath: `roletect_backup_${dateStr}_${timeStr}.json`
    });
    
    if (path) {
      await writeTextFile(path, JSON.stringify(data, null, 2));
    }
  } catch (error: any) {
    saveError.value = error.toString();
  } finally {
    isExporting.value = false;
  }
};

const selectedLocalBackupPath = ref<string>('');

const handleBrowseLocalBackup = async () => {
  try {
    const path = await openDialog({
      filters: [{ name: 'JSON Backup', extensions: ['json'] }],
      multiple: false
    });
    if (path && typeof path === 'string') {
      selectedLocalBackupPath.value = path;
    }
  } catch (error: any) {
    saveError.value = `File Picker Error: ${error.toString()}`;
  }
};

const handleImport = async (mode: 'merge' | 'overwrite') => {
  let targetPath = selectedLocalBackupPath.value;
  if (!targetPath) {
    const path = await openDialog({
      filters: [{ name: 'JSON Backup', extensions: ['json'] }],
      multiple: false
    });
    if (!path || typeof path !== 'string') return;
    targetPath = path;
    selectedLocalBackupPath.value = targetPath;
  }

  if (mode === 'overwrite') {
    const confirmed = await ask(
      'Are you absolutely sure you want to overwrite your entire local vault? This action cannot be undone and will permanently delete any unsaved local changes.',
      { title: 'Overwrite Local Vault?', kind: 'warning' }
    );
    if (!confirmed) return;
  }

  isImporting.value = true;
  try {
    const content = await readTextFile(targetPath);
    let raw = content;
    if (raw.charCodeAt(0) === 0xfeff) {
      raw = raw.slice(1);
    }
    let data: any;
    try {
      data = JSON.parse(raw);
    } catch {
      // If client-side JSON.parse failed, pass raw string to Rust backend parser
      data = raw;
    }
    await invoke('import_data', { data, mode });
    await dialog.showAlert(`Successfully ${mode === 'merge' ? 'synchronized' : 'restored'} your vault. The application will now reload to apply changes.`, 'Import Successful');
    window.location.reload();
  } catch (error: any) {
    saveError.value = `Import Error: ${error.toString()}`;
    await dialog.showAlert(`Failed to import vault: ${error.toString()}`, 'Import Error');
  } finally {
    isImporting.value = false;
  }
};

// --- S3 Backup State ---
const s3Endpoint = ref('');
const s3Bucket = ref('');
const s3Region = ref('us-east-1');
const s3AccessKey = ref('');
const hasS3AccessKey = ref(false);
const hasS3SecretKey = ref(false);
const s3SecretKey = ref('');
const s3ForcePathStyle = ref(true);
const isTestingS3 = ref(false);
const s3TestError = ref('');
const s3TestSuccess = ref(false);
const isSavingS3 = ref(false);
const s3SetupOk = ref(false);
const s3LastUpload = ref('Never');

const autoCloudBackup = ref(true);
const autoLocalBackup = ref(true);

const handleTestS3 = async (silent = false) => {
  if (!silent) {
    isTestingS3.value = true;
    s3TestError.value = '';
    s3TestSuccess.value = false;
  }
  
  try {
    let ak = s3AccessKey.value.trim();
    if (!ak) ak = await store.getSecret('s3_access_key') || '';
    
    let sk = s3SecretKey.value.trim();
    if (!sk) sk = await store.getSecret('s3_secret_key') || '';
    
    const res = await invoke<string>('test_s3_connection', {
      endpointUrl: s3Endpoint.value.trim(),
      bucketName: s3Bucket.value.trim(),
      region: s3Region.value.trim(),
      accessKeyId: ak,
      secretAccessKey: sk,
      forcePathStyle: s3ForcePathStyle.value
    });
    
    if (!silent) {
      s3TestSuccess.value = true;
      await dialog.showAlert(res, 'S3 Connection Succeeded');
    }
    return true;
  } catch (err: any) {
    if (!silent) s3TestError.value = err.toString();
    return false;
  } finally {
    if (!silent) isTestingS3.value = false;
  }
};

const handleSaveS3 = async () => {
  isSavingS3.value = true;
  s3TestError.value = '';
  s3TestSuccess.value = false;
  
  try {
    // Implicit test on save
    const testPassed = await handleTestS3(true);
    if (!testPassed) {
      s3SetupOk.value = false;
      await invoke('save_setting', { key: 's3_setup_ok', value: 'false' });
      throw new Error('Connection test failed. Please verify credentials/endpoint.');
    }
    
    await invoke('save_setting', { key: 's3_endpoint_url', value: s3Endpoint.value.trim() });
    await invoke('save_setting', { key: 's3_bucket_name', value: s3Bucket.value.trim() });
    await invoke('save_setting', { key: 's3_region', value: s3Region.value.trim() });
    await invoke('save_setting', { key: 's3_force_path_style', value: s3ForcePathStyle.value ? 'true' : 'false' });
    await invoke('save_setting', { key: 's3_setup_ok', value: 'true' });
    s3SetupOk.value = true;
    
    if (s3AccessKey.value.trim()) {
      await store.saveSecret('s3_access_key', s3AccessKey.value.trim());
    }
    if (s3SecretKey.value.trim()) {
      await store.saveSecret('s3_secret_key', s3SecretKey.value.trim());
    }
    
    s3TestSuccess.value = true;
    setTimeout(() => { s3TestSuccess.value = false; }, 3000);
  } catch (err: any) {
    s3TestError.value = err.toString();
  } finally {
    isSavingS3.value = false;
  }
};

interface BackupEntry {
  key: string;
  size: number;
  last_modified: string;
}

const isFetchingBackups = ref(false);
const fetchBackupsError = ref('');
const availableBackups = ref<BackupEntry[]>([]);
const selectedBackup = ref('');
const restoreMode = ref('merge');
const isRestoringBackup = ref(false);

const availableBackupsOptions = computed(() => {
  return availableBackups.value.map(b => ({
    value: b.key,
    label: `${b.key} (${Math.round(b.size / 1024)} KB)`
  }));
});

const restoreModeOptions = [
  { value: 'merge', label: 'Merge (Keep Existing Local Data)' },
  { value: 'overwrite', label: 'Overwrite All Local Data' }
];

const handleFetchBackups = async () => {
  isFetchingBackups.value = true;
  fetchBackupsError.value = '';
  availableBackups.value = [];
  selectedBackup.value = '';
  
  try {
    let ak = await store.getSecret('s3_access_key') || '';
    let sk = await store.getSecret('s3_secret_key') || '';
    
    const backups = await invoke<BackupEntry[]>('list_s3_backups', {
      accessKeyId: ak,
      secretAccessKey: sk
    });
    
    // Sort descending by last_modified
    availableBackups.value = backups.sort((a, b) => b.last_modified.localeCompare(a.last_modified));
    if (availableBackups.value.length > 0) {
      selectedBackup.value = availableBackups.value[0].key;
    } else {
      fetchBackupsError.value = 'No backups found in the bucket.';
    }
  } catch (err: any) {
    fetchBackupsError.value = err.toString();
  } finally {
    isFetchingBackups.value = false;
  }
};

const handleRestoreBackup = async () => {
  if (!selectedBackup.value) return;
  
  const modeText = restoreMode.value === 'overwrite' ? 'OVERWRITE ALL LOCAL DATA' : 'merge the cloud backup into your local data';
  
  const confirmed = await ask(
    `Are you sure you want to restore "${selectedBackup.value}"?\n\nWARNING: This will ${modeText}. The application will reload upon success.`,
    { title: 'Restore from Cloud Backup?', kind: 'warning' }
  );
  
  if (!confirmed) return;
  
  isRestoringBackup.value = true;
  try {
    let ak = await store.getSecret('s3_access_key') || '';
    let sk = await store.getSecret('s3_secret_key') || '';
    
    await invoke('restore_from_s3', {
      accessKeyId: ak,
      secretAccessKey: sk,
      key: selectedBackup.value,
      mode: restoreMode.value
    });
    
    await dialog.showAlert('Restore successful! The application will now reload to apply the restored data.', 'Restore Complete');
    window.location.reload();
  } catch (err: any) {
    await dialog.showAlert(err.toString(), 'Restore Failed');
  } finally {
    isRestoringBackup.value = false;
  }
};

// removed duplicate clearDataDialog

// --- 2. Configuration Data ---
const providers = [
  { id: 'openai', name: 'OpenAI' },
  { id: 'gemini', name: 'Google Gemini' },
  { id: 'anthropic', name: 'Anthropic Claude' },
  { id: 'groq', name: 'Groq (Ultra-Fast)' },
  { id: 'bedrock', name: 'AWS Bedrock' },
  { id: 'ollama', name: 'Ollama (Local LLM)' },
  { id: 'openrouter', name: 'OpenRouter' },
  { id: 'deepseek', name: 'DeepSeek' }
];

const modelsByProvider: Record<string, {id: string, name: string}[]> = {
  openrouter: [
    // --- 2026 Next-Gen Flagships ---
    { id: 'openai/gpt-6-astra', name: 'GPT-6 Astra' },
    { id: 'openai/gpt-5.6-sol', name: 'GPT-5.6 Sol' },
    { id: 'openai/gpt-5.4-thinking', name: 'GPT-5.4 Thinking' },
    { id: 'openai/gpt-4.1', name: 'GPT-4.1' },
    { id: 'anthropic/claude-opus-5', name: 'Claude Opus 5' },
    { id: 'anthropic/claude-sonnet-5', name: 'Claude Sonnet 5' },
    { id: 'google/gemini-3.8-flash', name: 'Gemini 3.8 Flash' },
    { id: 'google/gemini-3.7-flash', name: 'Gemini 3.7 Flash' },
    { id: 'google/gemini-3.6-flash', name: 'Gemini 3.6 Flash' },
    { id: 'deepseek/deepseek-v4-pro-0813', name: 'DeepSeek V4 Pro (0813)' },
    { id: 'deepseek/deepseek-v4-flash-0731', name: 'DeepSeek V4 Flash (0731)' },

    // --- Gemini 3.x and 2.5 (Text & Reasoning) ---
    { id: 'google/gemini-3.1-pro-preview', name: 'Gemini 3.1 Pro (Preview)' },
    { id: 'google/gemini-3.1-flash-preview', name: 'Gemini 3.1 Flash (Preview)' },
    { id: 'google/gemini-3-flash-preview', name: 'Gemini 3.0 Flash (Preview)' },
    { id: 'google/gemini-3-deep-think', name: 'Gemini 3.0 Deep Think' },
    { id: 'google/gemini-2.5-pro', name: 'Gemini 2.5 Pro' },
    { id: 'google/gemini-2.5-flash', name: 'Gemini 2.5 Flash' },
    { id: 'google/gemini-2.0-flash', name: 'Gemini 2.0 Flash' },
    
    // --- Claude Models ---
    { id: 'anthropic/claude-4-opus', name: 'Claude 4 Opus' },
    { id: 'anthropic/claude-4-sonnet', name: 'Claude 4 Sonnet' },
    { id: 'anthropic/claude-3.7-sonnet', name: 'Claude 3.7 Sonnet' },
    { id: 'anthropic/claude-3.7-sonnet:thinking', name: 'Claude 3.7 Sonnet (Thinking)' },
    { id: 'anthropic/claude-3.5-sonnet', name: 'Claude 3.5 Sonnet' },
    { id: 'anthropic/claude-3.5-haiku', name: 'Claude 3.5 Haiku' },
    { id: 'anthropic/claude-3-opus', name: 'Claude 3 Opus' },
    { id: 'anthropic/claude-3-haiku', name: 'Claude 3 Haiku' },

    // --- DeepSeek Models ---
    { id: 'deepseek/deepseek-v4-pro', name: 'DeepSeek V4 Pro' },
    { id: 'deepseek/deepseek-v4-flash', name: 'DeepSeek V4 Flash' },
    { id: 'deepseek/deepseek-chat', name: 'DeepSeek V3' },
    { id: 'deepseek/deepseek-r1', name: 'DeepSeek R1' },
    { id: 'deepseek/deepseek-r1:free', name: 'DeepSeek R1 (Free)' },
    { id: 'deepseek/deepseek-chat:free', name: 'DeepSeek V3 (Free)' },

    // --- Llama Models ---
    { id: 'meta-llama/llama-4-scout-17b-16e-instruct', name: 'Llama 4 Scout (17B x 16E)' },
    { id: 'meta-llama/llama-4-maverick-17b-128e-instruct', name: 'Llama 4 Maverick (17B x 128E)' },
    { id: 'meta-llama/llama-4-maverick-400b-instruct', name: 'Llama 4 Maverick (400B)' },
    { id: 'meta-llama/llama-3.3-70b-instruct', name: 'Llama 3.3 70B Instruct' },
    { id: 'meta-llama/llama-3.1-405b-instruct', name: 'Llama 3.1 405B Instruct' },
    { id: 'meta-llama/llama-3.1-70b-instruct', name: 'Llama 3.1 70B Instruct' },
    { id: 'meta-llama/llama-3.1-8b-instruct', name: 'Llama 3.1 8B Instruct' },
    { id: 'meta-llama/llama-3.2-3b-instruct', name: 'Llama 3.2 3B Instruct' },

    // --- Qwen & Mistral Models ---
    { id: 'qwen/qwen-3-32b', name: 'Qwen3 32B' },
    { id: 'qwen/qwen-2.5-coder-32b-instruct', name: 'Qwen 2.5 Coder 32B' },
    { id: 'qwen/qwen-2.5-72b-instruct', name: 'Qwen 2.5 72B Instruct' },
    { id: 'qwen/qwen-2.5-7b-instruct', name: 'Qwen 2.5 7B' },
    { id: 'mistralai/mistral-large', name: 'Mistral Large 2411' },
    { id: 'mistralai/mistral-7b-instruct', name: 'Mistral 7B Instruct' },

    // --- OpenAI Models ---
    { id: 'openai/gpt-5-main', name: 'GPT-5 Foundation' },
    { id: 'openai/gpt-5-mini', name: 'GPT-5 Mini' },
    { id: 'openai/gpt-5.5', name: 'GPT-5.5' },
    { id: 'openai/gpt-5.5-thinking', name: 'GPT-5.5 Thinking' },
    { id: 'openai/o3', name: 'o3 (Reasoning)' },
    { id: 'openai/o3-mini', name: 'o3 Mini' },
    { id: 'openai/o3-mini:high', name: 'o3 Mini (High Reasoning)' },
    { id: 'openai/o4-mini', name: 'o4 Mini' },
    { id: 'openai/o1', name: 'o1' },
    { id: 'openai/o1-mini', name: 'o1 Mini' },
    { id: 'openai/gpt-4o', name: 'GPT-4o' },
    { id: 'openai/gpt-4o-mini', name: 'GPT-4o Mini' },
    { id: 'openai/gpt-4-turbo', name: 'GPT-4 Turbo' }
  ],
  openai: [
    // --- 2026 Next-Gen Flagships ---
    { id: 'gpt-6-astra', name: 'GPT-6 Astra (Flagship)' },
    { id: 'gpt-5.6-sol', name: 'GPT-5.6 Sol' },
    { id: 'gpt-5.4-thinking', name: 'GPT-5.4 Thinking' },
    { id: 'gpt-4.1', name: 'GPT-4.1' },

    // --- GPT-5 Era ---
    { id: 'gpt-5-nano', name: 'GPT-5 Nano' },
    { id: 'gpt-5-mini', name: 'GPT-5 Mini' },
    { id: 'gpt-5-main', name: 'GPT-5 Foundation' },
    { id: 'gpt-5.1', name: 'GPT-5.1' },
    { id: 'gpt-5.2', name: 'GPT-5.2' },
    { id: 'gpt-5.3-instant', name: 'GPT-5.3 Instant' },
    { id: 'gpt-5.3-codex-spark', name: 'GPT-5.3 Codex Spark (Coding)' },
    { id: 'gpt-5.4-nano', name: 'GPT-5.4 Nano' },
    { id: 'gpt-5.4-mini', name: 'GPT-5.4 Mini' },
    { id: 'gpt-5.4', name: 'GPT-5.4 Standard' },
    { id: 'gpt-5.5-instant', name: 'GPT-5.5 Instant' },
    { id: 'gpt-5.5', name: 'GPT-5.5' },
    { id: 'gpt-5.5-thinking', name: 'GPT-5.5 Thinking' },
    { id: 'gpt-5.5-pro', name: 'GPT-5.5 Pro (Thinking)' },
    { id: 'gpt-rosalind', name: 'GPT-Rosalind (Life Sciences)' },

    // --- Reasoning Models ---
    { id: 'o1', name: 'o1' },
    { id: 'o1-mini', name: 'o1 Mini' },
    { id: 'o1-preview', name: 'o1 Preview' },
    { id: 'o3', name: 'o3 (Reasoning)' },
    { id: 'o3-mini', name: 'o3 Mini' },
    { id: 'o4-mini', name: 'o4 Mini' },

    // --- Legacy / Classic ---
    { id: 'gpt-4o', name: 'GPT-4o (Omni)' },
    { id: 'gpt-4o-mini', name: 'GPT-4o Mini' },
    { id: 'gpt-4-turbo', name: 'GPT-4 Turbo' },
    { id: 'gpt-4', name: 'GPT-4' },
    { id: 'gpt-3.5-turbo', name: 'GPT-3.5 Turbo' }
  ],
  
  gemini: [
    // --- 2026 Gemini 3.x Releases ---
    { id: 'gemini-3.8-flash', name: 'Gemini 3.8 Flash' },
    { id: 'gemini-3.7-flash', name: 'Gemini 3.7 Flash' },
    { id: 'gemini-3.6-flash', name: 'Gemini 3.6 Flash' },
    { id: 'gemini-3.1-flash-preview', name: 'Gemini 3.1 Flash (Preview)' },
    { id: 'gemini-3.1-pro-preview', name: 'Gemini 3.1 Pro (Preview)' },
    { id: 'gemini-3.1-flash-lite', name: 'Gemini 3.1 Flash-Lite (Stable/GA)' },
    { id: 'gemini-3.1-flash-lite-preview', name: 'Gemini 3.1 Flash-Lite (Preview)' },
    { id: 'gemini-3-flash-preview', name: 'Gemini 3.0 Flash (Preview)' },
    { id: 'gemini-3-deep-think', name: 'Gemini 3.0 Deep Think' },
    { id: 'gemini-3-pro-preview', name: 'Gemini 3.0 Pro (Preview)' },

    // --- Gemini 2.5 & 2.0 Era ---
    { id: 'gemini-2.5-flash-lite', name: 'Gemini 2.5 Flash-Lite' },
    { id: 'gemini-2.5-flash', name: 'Gemini 2.5 Flash (Stable)' },
    { id: 'gemini-2.5-pro', name: 'Gemini 2.5 Pro (Stable)' },
    { id: 'gemini-2.0-flash-lite', name: 'Gemini 2.0 Flash-Lite' },
    { id: 'gemini-2.0-flash', name: 'Gemini 2.0 Flash' },
    { id: 'gemini-2.0-pro', name: 'Gemini 2.0 Pro' },
    { id: 'gemini-1.5-flash', name: 'Gemini 1.5 Flash' },
    { id: 'gemini-1.5-pro', name: 'Gemini 1.5 Pro' },
    
    // --- Deep Research & Open Weights Text ---
    { id: 'deep-research-preview-04-2026', name: 'Deep Research Agent (Preview)' },
    { id: 'deep-research-max-preview-04-2026', name: 'Deep Research Max (Preview)' },
    { id: 'gemma-4-31b-it', name: 'Gemma 4 31B IT' },
    { id: 'gemma-4-26b-a4b-it', name: 'Gemma 4 26B IT' },
    { id: 'gemma-3-27b-it', name: 'Gemma 3 27B IT' },
    { id: 'gemma-2-9b-it', name: 'Gemma 2 9B IT' }
  ],
  
  anthropic: [
    // --- 2026 Claude 5 & 4.5 Releases ---
    { id: 'claude-opus-5', name: 'Claude Opus 5 (Flagship)' },
    { id: 'claude-sonnet-5', name: 'Claude Sonnet 5' },
    { id: 'claude-haiku-4-5', name: 'Claude Haiku 4.5' },
    { id: 'claude-opus-4-7', name: 'Claude Opus 4.7 (Power)' },
    { id: 'claude-opus-4-6', name: 'Claude Opus 4.6' },
    { id: 'claude-opus-4-5', name: 'Claude Opus 4.5' },
    { id: 'claude-sonnet-4-6', name: 'Claude Sonnet 4.6 (Balanced)' },
    { id: 'claude-sonnet-4-5', name: 'Claude Sonnet 4.5' },
    { id: 'claude-4-sonnet', name: 'Claude 4 Sonnet' },
    { id: 'claude-4-opus', name: 'Claude 4 Opus' },
    { id: 'claude-4-1-opus', name: 'Claude 4.1 Opus' },

    // --- Claude 3 Era ---
    { id: 'claude-3-7-sonnet-latest', name: 'Claude 3.7 Sonnet' },
    { id: 'claude-3-5-sonnet-latest', name: 'Claude 3.5 Sonnet' },
    { id: 'claude-3-5-haiku-latest', name: 'Claude 3.5 Haiku' },
    { id: 'claude-3-haiku-20240307', name: 'Claude 3 Haiku' },
    { id: 'claude-3-sonnet-20240229', name: 'Claude 3 Sonnet' },
    { id: 'claude-3-opus-20240229', name: 'Claude 3 Opus' },
    
    // --- Agents & Specialized ---
    { id: 'claude-cowork', name: 'Claude Cowork (Multi-step Agent)' },
    { id: 'claude-mythos-preview', name: 'Claude Mythos Preview (Security)' }
  ],
  
  groq: [
    // --- LPU Accelerated Flagships ---
    { id: 'llama-3.3-70b-versatile', name: 'Llama 3.3 70B Versatile' },
    { id: 'openai/gpt-oss-120b', name: 'GPT-OSS 120B (Groq LPU Flagship)' },
    { id: 'openai/gpt-oss-20b', name: 'GPT-OSS 20B' },
    { id: 'openai/gpt-oss-safeguard-20b', name: 'GPT-OSS Safeguard 20B' },

    // --- Llama 4 Era (Current) ---
    { id: 'meta-llama/llama-4-scout-17b-16e-instruct', name: 'Llama 4 Scout (17B x 16E)' },
    { id: 'meta-llama/llama-4-maverick-17b-128e-instruct', name: 'Llama 4 Maverick (17B x 128E)' },
    { id: 'meta-llama/llama-4-maverick-400b-instruct', name: 'Llama 4 Maverick (400B)' },

    // --- Llama 3.x Era ---
    { id: 'llama-3.1-8b-instant', name: 'Llama 3.1 8B Instant' },
    { id: 'llama-3.1-70b-versatile', name: 'Llama 3.1 70B Versatile' },
    { id: 'llama-3.3-70b-specdec', name: 'Llama 3.3 SpecDec' },

    // --- Mistral ---
    { id: 'mistral-medium-3.5', name: 'Mistral Medium 3.5' },
    { id: 'mistral-small-4', name: 'Mistral Small 4' },
    { id: 'mixtral-8x7b-32768', name: 'Mixtral 8x7B' },

    // --- DeepSeek, Qwen & Others ---
    { id: 'deepseek-v4-pro-0813', name: 'DeepSeek V4 Pro (0813)' },
    { id: 'deepseek-v4-flash-0731', name: 'DeepSeek V4 Flash (0731)' },
    { id: 'deepseek-v4-flash', name: 'DeepSeek-V4 Flash' },
    { id: 'deepseek-v4-pro', name: 'DeepSeek-V4 Pro' },
    { id: 'deepseek-r1-distill-llama-70b', name: 'DeepSeek R1 Distill Llama 70B' },
    { id: 'qwen-3-32b', name: 'Qwen3 32B' },
    { id: 'qwen/qwen-2.5-coder-32b-instruct', name: 'Qwen 2.5 Coder 32B' },
    { id: 'glm-5.1', name: 'GLM-5.1' },
    { id: 'moonshotai/kimi-k2-instruct-0905', name: 'Kimi K2 Instruct (Reasoning)' },
    { id: 'minimaxai/minimax-m2.5', name: 'Minimax M2.5 (Enterprise)' }
  ],
  
  bedrock: [
    // --- Deep Reasoning & Thinking (Heavy Logic, Code, Math) ---
    { id: 'deepseek.r1-v1:0', name: 'DeepSeek R1' },
    { id: 'anthropic.claude-opus-5', name: 'Claude Opus 5' },
    { id: 'anthropic.claude-opus-4-8', name: 'Claude Opus 4.8' },
    { id: 'anthropic.claude-opus-4-7', name: 'Claude Opus 4.7' },
    { id: 'anthropic.claude-opus-4-6', name: 'Claude Opus 4.6' },
    { id: 'anthropic.claude-opus-4-5', name: 'Claude Opus 4.5' },
    { id: 'openai.gpt-6-astra', name: 'GPT-6 Astra' },
    { id: 'openai.gpt-5-5-v1:0', name: 'GPT-5.5' },
    { id: 'openai.gpt-5-4-v1:0', name: 'GPT-5.4' },
    { id: 'moonshot.kimi-k2-thinking-v1:0', name: 'Kimi K2 Thinking' },
    { id: 'mistral.mistral-large-2407-v1:0', name: 'Mistral Large 3' },
    { id: 'mistral.devstral-2-123b-v1:0', name: 'Devstral 2 123B' },
    { id: 'qwen.qwen3-coder-next-v1:0', name: 'Qwen3 Coder Next' },
    { id: 'ai21.jamba-1-5-large-v1:0', name: 'Jamba 1.5 Large' },

    // --- General Purpose & Balanced (Text & Chat) ---
    { id: 'anthropic.claude-sonnet-5', name: 'Claude Sonnet 5' },
    { id: 'anthropic.claude-sonnet-4-6', name: 'Claude Sonnet 4.6' },
    { id: 'anthropic.claude-sonnet-4-5', name: 'Claude Sonnet 4.5' },
    { id: 'anthropic.claude-3-7-sonnet-20250219-v1:0', name: 'Claude 3.7 Sonnet' },
    { id: 'anthropic.claude-3-5-sonnet-20241022-v2:0', name: 'Claude 3.5 Sonnet v2' },
    { id: 'amazon.nova-pro-v1:0', name: 'Amazon Nova Pro' },
    { id: 'meta.llama4-maverick-17b-instruct-v1:0', name: 'Llama 4 Maverick 17B Instruct' },
    { id: 'meta.llama4-scout-17b-instruct-v1:0', name: 'Llama 4 Scout 17B Instruct' },
    { id: 'meta.llama3-3-70b-instruct-v1:0', name: 'Llama 3.3 70B Instruct' },
    { id: 'z-ai.glm-4-7-v1:0', name: 'GLM 4.7' },
    { id: 'moonshot.kimi-k2-5-v1:0', name: 'Kimi K2.5' },
    { id: 'minimax.m2-5-v1:0', name: 'MiniMax M2.5' },

    // --- Light & Fast (High Throughput Text) ---
    { id: 'anthropic.claude-haiku-4-5', name: 'Claude Haiku 4.5' },
    { id: 'anthropic.claude-3-5-haiku-20241022-v1:0', name: 'Claude 3.5 Haiku' },
    { id: 'amazon.nova-2-lite-v1:0', name: 'Amazon Nova 2 Lite' },
    { id: 'amazon.nova-micro-v1:0', name: 'Amazon Nova Micro' },
    { id: 'deepseek.v3-2-v1:0', name: 'DeepSeek V3.2' },
    { id: 'deepseek.v3-1-v1:0', name: 'DeepSeek V3.1' },
    { id: 'mistral.ministral-3-8b-v1:0', name: 'Ministral 3 8B' },
    { id: 'z-ai.glm-4-7-flash-v1:0', name: 'GLM 4.7 Flash' },
    { id: 'ai21.jamba-1-5-mini-v1:0', name: 'Jamba 1.5 Mini' },
    { id: 'nvidia.nemotron-3-super-120b-v1:0', name: 'NVIDIA Nemotron 3 Super 120B' }
  ],
  ollama: [
    // --- Llama Models ---
    { id: 'llama3.3', name: 'Llama 3.3 (70B)' },
    { id: 'llama3.2', name: 'Llama 3.2 (3B)' },
    { id: 'llama3.2:1b', name: 'Llama 3.2 (1B)' },
    { id: 'llama3.1', name: 'Llama 3.1 (8B)' },
    { id: 'llama3', name: 'Llama 3 (8B)' },

    // --- DeepSeek Reasoning & Chat ---
    { id: 'deepseek-r1:70b', name: 'DeepSeek R1 (70B)' },
    { id: 'deepseek-r1:32b', name: 'DeepSeek R1 (32B)' },
    { id: 'deepseek-r1:14b', name: 'DeepSeek R1 (14B)' },
    { id: 'deepseek-r1:8b', name: 'DeepSeek R1 (8B)' },
    { id: 'deepseek-r1:7b', name: 'DeepSeek R1 (7B)' },
    { id: 'deepseek-r1:1.5b', name: 'DeepSeek R1 (1.5B)' },
    { id: 'deepseek-v3', name: 'DeepSeek V3' },
    { id: 'deepseek-coder-v2', name: 'DeepSeek Coder V2' },

    // --- Qwen & Qwen Coder ---
    { id: 'qwen2.5-coder:32b', name: 'Qwen 2.5 Coder (32B)' },
    { id: 'qwen2.5-coder:14b', name: 'Qwen 2.5 Coder (14B)' },
    { id: 'qwen2.5-coder:7b', name: 'Qwen 2.5 Coder (7B)' },
    { id: 'qwen2.5-coder:1.5b', name: 'Qwen 2.5 Coder (1.5B)' },
    { id: 'qwen2.5-coder', name: 'Qwen 2.5 Coder' },
    { id: 'qwen2.5:72b', name: 'Qwen 2.5 (72B)' },
    { id: 'qwen2.5:32b', name: 'Qwen 2.5 (32B)' },
    { id: 'qwen2.5:14b', name: 'Qwen 2.5 (14B)' },
    { id: 'qwen2.5:7b', name: 'Qwen 2.5 (7B)' },

    // --- Mistral, Gemma & Phi ---
    { id: 'mistral-small', name: 'Mistral Small' },
    { id: 'mistral-nemo', name: 'Mistral Nemo (12B)' },
    { id: 'mistral', name: 'Mistral (7B)' },
    { id: 'gemma2:27b', name: 'Gemma 2 (27B)' },
    { id: 'gemma2', name: 'Gemma 2 (9B)' },
    { id: 'gemma2:2b', name: 'Gemma 2 (2B)' },
    { id: 'phi4', name: 'Phi 4 (14B)' },
    { id: 'phi3', name: 'Phi 3 (3.8B)' }
  ],
  deepseek: [
    // --- 2026 DeepSeek V4 Releases ---
    { id: 'deepseek-v4-pro-0813', name: 'DeepSeek V4 Pro (0813)' },
    { id: 'deepseek-v4-flash-0731', name: 'DeepSeek V4 Flash (0731)' },
    { id: 'deepseek-v4-pro', name: 'DeepSeek V4 Pro' },
    { id: 'deepseek-v4-flash', name: 'DeepSeek V4 Flash' },
    { id: 'deepseek-chat', name: 'DeepSeek V3 (Chat)' },
    { id: 'deepseek-reasoner', name: 'DeepSeek R1' },
    { id: 'deepseek-coder-v3', name: 'DeepSeek Coder V3' }
  ]
};

// --- 3. Logic & Helpers ---

// Check if the current draft differs from the saved store
const hasChanges = computed(() => {
  return (
    providerInput.value !== store.selectedAiProvider ||
    modelInput.value !== store.selectedAiModel ||
    apiKeyInput.value.length > 0 ||
    customBaseUrlInput.value !== savedCustomBaseUrl.value ||
    customModelInput.value !== savedCustomModel.value
  );
});

const providerName = computed(() => 
  providers.find(p => p.id === providerInput.value)?.name || 'AI'
);

const currentModels = computed(() => 
  modelsByProvider[providerInput.value] || []
);

// Resets local UI state to match the DB
const syncFromStore = async () => {
  await store.loadSettings();
  providerInput.value = store.selectedAiProvider;
  modelInput.value = store.selectedAiModel;
  apiKeyInput.value = ''; // Reset the input buffer

  const url = await invoke('get_setting', { key: `${providerInput.value}_custom_base_url`, defaultValue: '' }) as string;
  const customModel = await invoke('get_setting', { key: `${providerInput.value}_custom_model`, defaultValue: '' }) as string;
  
  customBaseUrlInput.value = url;
  customModelInput.value = customModel;
  savedCustomBaseUrl.value = url;
  savedCustomModel.value = customModel;

  await store.loadProviderKeyStatus(providerInput.value);

  // Load S3 settings
  s3Endpoint.value = await invoke('get_setting', { key: 's3_endpoint_url', defaultValue: '' }) as string;
  s3Bucket.value = await invoke('get_setting', { key: 's3_bucket_name', defaultValue: '' }) as string;
  s3Region.value = await invoke('get_setting', { key: 's3_region', defaultValue: 'us-east-1' }) as string;
  s3ForcePathStyle.value = await invoke('get_setting', { key: 's3_force_path_style', defaultValue: 'true' }) === 'true';
  s3SetupOk.value = await invoke('get_setting', { key: 's3_setup_ok', defaultValue: 'false' }) === 'true';
  s3LastUpload.value = await invoke('get_setting', { key: 's3_last_upload', defaultValue: 'Never' }) as string;
  s3AccessKey.value = '';
  s3SecretKey.value = '';
  
  const s3Ak = await store.getSecret('s3_access_key');
  const s3Sk = await store.getSecret('s3_secret_key');
  hasS3AccessKey.value = !!s3Ak;
  hasS3SecretKey.value = !!s3Sk;
  
  autoCloudBackup.value = (await invoke<string>('get_setting', { key: 'auto_cloud_backup', defaultValue: 'true' })) === 'true';
  autoLocalBackup.value = (await invoke<string>('get_setting', { key: 'auto_local_backup', defaultValue: 'true' })) === 'true';
};

const scrollToActivation = () => {
  const el = document.getElementById('license-activation');
  if (el) {
    el.scrollIntoView({ behavior: 'smooth', block: 'center' });
    const input = document.getElementById('upgrade-key-input') as HTMLInputElement | null;
    if (input) {
      setTimeout(() => {
        input.focus();
        input.classList.add('pulse-focus');
        setTimeout(() => input.classList.remove('pulse-focus'), 1600);
      }, 350);
    }
  }
};

onMounted(async () => {
  await syncFromStore();
  if (route.hash === '#license-activation' || route.hash === '#license') {
    setTimeout(scrollToActivation, 250);
  }
});

watch(() => route.hash, (newHash) => {
  if (newHash === '#license-activation' || newHash === '#license') {
    setTimeout(scrollToActivation, 100);
  }
});

watch(autoCloudBackup, (val) => {
  invoke('save_setting', { key: 'auto_cloud_backup', value: val ? 'true' : 'false' });
});

watch(autoLocalBackup, (val) => {
  invoke('save_setting', { key: 'auto_local_backup', value: val ? 'true' : 'false' });
});

// When provider changes, adjust the model but DON'T wipe the Store state yet
watch(providerInput, async (newProvider) => {
  if (!newProvider) return;
  const availableModels = modelsByProvider[newProvider];

  const url = await invoke('get_setting', { key: `${newProvider}_custom_base_url`, defaultValue: '' }) as string;
  const customModel = await invoke('get_setting', { key: `${newProvider}_custom_model`, defaultValue: '' }) as string;
  
  customBaseUrlInput.value = url;
  customModelInput.value = customModel;
  savedCustomBaseUrl.value = url;
  savedCustomModel.value = customModel;

  if (customModel) {
    modelInput.value = customModel;
  } else if (!availableModels.find(m => m.id === modelInput.value)) {
    modelInput.value = availableModels[0].id;
  }
  
  // Check if THIS specific provider has a key saved in Stronghold
  await store.loadProviderKeyStatus(newProvider);
});

const sortedThemes = computed(() => {
  return [...store.availableThemes].sort((a, b) => a.name.localeCompare(b.name));
});

const alertFeatureCapped = async (featureName: string) => {
  await dialog.showAlert(
    `${featureName} is a Pro feature that is capped in the Free Tier.\n\nAll core features (AI tailoring, LaTeX documents, compiler & vault) remain fully available. Activate your license to unlock full theme & typography customization.`,
    'Pro Feature'
  );
  scrollToActivation();
};

const handleThemeChange = async (val: string | Event) => {
  if (!licenseStore.isLicensed) {
    await alertFeatureCapped('Theme customization');
    return;
  }
  const actualVal = typeof val === 'string' ? val : (val.target as HTMLSelectElement).value;
  await store.setTheme(actualVal);
};

const handleImportTheme = async () => {
  if (!licenseStore.isLicensed) {
    await alertFeatureCapped('Theme import');
    return;
  }
  try {
    themeError.value = '';
    await store.importCustomTheme(customThemeJson.value);
    customThemeJson.value = '';
    isImportingTheme.value = false;
    await dialog.showAlert('Custom theme imported successfully.', 'Theme Imported');
  } catch (e: any) {
    themeError.value = e.message;
  }
};

const handleDeleteTheme = async (id: string) => {
  if (!licenseStore.isLicensed) {
    await alertFeatureCapped('Theme management');
    return;
  }
  const confirmed = await dialog.showConfirm('Are you sure you want to delete this custom theme?', 'Delete Theme');
  if (confirmed) {
    try {
      await store.deleteCustomTheme(id);
      await dialog.showAlert('Theme deleted successfully.', 'Theme Deleted');
    } catch (e: any) {
      saveError.value = e.toString();
    }
  }
};

const showThemeSchema = () => {
  const schema = `Theme JSON should follow this format:
{
  "name": "My Theme",
  "colors": {
    "--bg": "#...",
    "--bg-accent": "#...",
    ...
  }
}`;
  dialog.showAlert(schema, 'Theme Schema');
};

const handleFontFamilyChange = async (val: string | Event) => {
  if (!licenseStore.isLicensed) {
    await alertFeatureCapped('Font family customization');
    return;
  }
  const actualVal = typeof val === 'string' ? val : (val.target as HTMLSelectElement).value;
  await store.setFontFamily(actualVal);
};

const handleFontSizeChange = async (event: Event) => {
  if (!licenseStore.isLicensed) {
    await alertFeatureCapped('Font size customization');
    return;
  }
  const target = event.target as HTMLInputElement;
  await store.setFontSize(parseInt(target.value));
};

const handleFontWeightChange = async (val: string | Event) => {
  if (!licenseStore.isLicensed) {
    await alertFeatureCapped('Font weight customization');
    return;
  }
  const actualVal = typeof val === 'string' ? val : (val.target as HTMLSelectElement).value;
  await store.setFontWeight(actualVal);
};

const handleFontStyleChange = async (val: string | Event) => {
  if (!licenseStore.isLicensed) {
    await alertFeatureCapped('Font style customization');
    return;
  }
  const actualVal = typeof val === 'string' ? val : (val.target as HTMLSelectElement).value;
  await store.setFontStyle(actualVal);
};

const handleResetTypography = async () => {
  if (!licenseStore.isLicensed) {
    await alertFeatureCapped('Typography reset');
    return;
  }
  const confirmed = await dialog.showConfirm('Reset all typography settings to default?', 'Reset Typography');
  if (confirmed) {
    await store.resetTypography();
  }
};

const isTestingConnection = ref(false);

const handleTestConnection = async () => {
  isTestingConnection.value = true;
  saveError.value = '';
  
  try {
    let testKey = apiKeyInput.value.trim();
    if (testKey === '') {
      const decrypted = await store.getDecryptedKey(providerInput.value);
      testKey = decrypted || '';
    }
    
    const response = await invoke<string>('test_ai_connection', {
      provider: providerInput.value,
      model: modelInput.value,
      apiKey: testKey
    });
    
    let formattedResponse = response;
    try {
      const parsed = JSON.parse(response);
      formattedResponse = JSON.stringify(parsed, null, 2);
    } catch {
      // Ignore parsing error
    }
    
    await dialog.showAlert(`Response:\n${formattedResponse}`, 'AI Connection Succeeded');
  } catch (error: any) {
    await dialog.showAlert(error.toString(), 'AI Connection Failed');
  } finally {
    isTestingConnection.value = false;
  }
};

const handleSave = async () => {
  isSaving.value = true;
  saveError.value = '';
  
  try {
    // 1. If user typed a new key, save it
    if (apiKeyInput.value.trim() !== '') {
      await store.saveApiKey(providerInput.value, apiKeyInput.value.trim());
    }
    
    // 2. Save the provider/model choice, utilizing custom model if specified
    const finalModel = customModelInput.value.trim() !== '' ? customModelInput.value.trim() : modelInput.value;
    await store.saveModelConfig(
      providerInput.value, 
      finalModel, 
      customBaseUrlInput.value.trim(), 
      customModelInput.value.trim()
    );
    
    // 3. Re-sync everything and show success
    await syncFromStore();
    showSuccess.value = true;
    setTimeout(() => { showSuccess.value = false; }, 3000);
  } catch (error: any) {
    saveError.value = error.message || 'Failed to save configuration.';
  } finally {
    isSaving.value = false;
  }
};
</script>

<template>
  <div class="settings-container">
    <div class="header">
      <h2>Engine Configuration</h2>
      <p class="subtitle">Customize how the AI intelligence layer behaves.</p>
    </div>

    <div class="settings-grid">
      <!-- UI Customization -->
      <div class="settings-card">
        <div class="card-header">
          <div class="title-row">
            <div class="title-with-badge">
              <h3>Visual Persona</h3>
              <span v-if="!licenseStore.isLicensed" class="capped-badge">
                <Lock :size="11" /> Capped (Free Tier)
              </span>
            </div>
            <div class="header-btns">
              <button class="text-btn secondary" :disabled="!licenseStore.isLicensed" @click="store.setTheme('github-dark')">
                <RotateCcw :size="14" /> Reset
              </button>
              <button class="text-btn secondary" :disabled="!licenseStore.isLicensed" @click="copyDemoTheme">
                <Download :size="14" /> Copy Demo
              </button>
              <button class="text-btn" :disabled="!licenseStore.isLicensed" @click="isImportingTheme = !isImportingTheme">
                <Plus :size="14" /> {{ isImportingTheme ? 'Cancel' : 'Import Theme' }}
              </button>
            </div>
          </div>
          <p>Choose a premium built-in theme or import your own surgical palette.</p>
        </div>

        <div v-if="!licenseStore.isLicensed" class="feature-capped-banner">
          <div class="banner-content">
            <Lock :size="15" class="banner-icon" />
            <div class="banner-text">
              <strong>Theme Customization is Locked</strong>
              <p>The free tier uses the GitHub Dark theme. All core features (AI tailoring, LaTeX documents, tectonic compiler & local vault) are completely free. Upgrade to customize or import surgical themes.</p>
            </div>
          </div>
          <button class="btn-activate-inline" @click="scrollToActivation">
            <Sparkles :size="13" /> Activate License
          </button>
        </div>

        <div class="theme-selector-row">
          <div class="input-group">
            <label>Active Theme</label>
            <div class="theme-picker-wrapper">
              <CustomSelect 
                :model-value="store.activeThemeId" 
                :disabled="!licenseStore.isLicensed"
                @change="handleThemeChange" 
                :options="sortedThemes.map(theme => ({ value: theme.id, label: theme.name + (theme.is_builtin ? ' (Built-in)' : '') }))"
              >
                <template #icon>
                  <Palette :size="16" style="color: var(--accent);" />
                </template>
              </CustomSelect>
            </div>
          </div>
          
          <button 
            v-if="!store.availableThemes.find(t => t.id === store.activeThemeId)?.is_builtin"
            class="delete-theme-btn"
            :disabled="!licenseStore.isLicensed"
            @click="handleDeleteTheme(store.activeThemeId)"
          >
            <Trash2 :size="16" />
          </button>
        </div>

        <AnimatePresence>
          <Motion
            v-if="isImportingTheme"
            :initial="{ height: 0, opacity: 0 }"
            :animate="{ height: 'auto', opacity: 1 }"
            :exit="{ height: 0, opacity: 0 }"
            class="import-theme-area"
          >
            <div class="import-header">
              <label>Theme JSON Configuration</label>
              <button class="help-link-btn" @click.prevent="showThemeSchema">View Schema</button>
            </div>
            <textarea 
              v-model="customThemeJson" 
              placeholder='{ "name": "Deep Ocean", "colors": { "--bg": "#000b1e", ... } }'
              class="theme-textarea"
            ></textarea>
            <div class="import-actions-row">
              <span v-if="themeError" class="error-inline">
                <span>{{ themeError }}</span>
                <button
                  type="button"
                  class="copy-err-inline-btn"
                  @click="handleCopyInlineError(themeError)"
                  :title="copiedInlineError === themeError ? 'Copied!' : 'Copy Error'"
                >
                  <Check v-if="copiedInlineError === themeError" :size="11" />
                  <Copy v-else :size="11" />
                  <span>{{ copiedInlineError === themeError ? 'Copied!' : 'Copy' }}</span>
                </button>
              </span>
              <button class="btn-import-confirm" @click="handleImportTheme">Import & Apply</button>
            </div>
          </Motion>
        </AnimatePresence>
      </div>

      <!-- Typography Settings -->
      <div class="settings-card">
        <div class="card-header">
          <div class="title-row">
            <div class="title-with-badge">
              <h3>Typography</h3>
              <span v-if="!licenseStore.isLicensed" class="capped-badge">
                <Lock :size="11" /> Capped (Free Tier)
              </span>
            </div>
            <button class="text-btn secondary" :disabled="!licenseStore.isLicensed" @click="handleResetTypography">
              <RotateCcw :size="14" /> Reset
            </button>
          </div>
          <p>Adjust the interface fonts to suit your surgical workflow.</p>
        </div>

        <div v-if="!licenseStore.isLicensed" class="feature-capped-banner">
          <div class="banner-content">
            <Lock :size="15" class="banner-icon" />
            <div class="banner-text">
              <strong>Typography Customization is Locked</strong>
              <p>The free tier uses standard Inter typography. Upgrade to personalize font families, weights, styles, and interface scale.</p>
            </div>
          </div>
          <button class="btn-activate-inline" @click="scrollToActivation">
            <Sparkles :size="13" /> Activate License
          </button>
        </div>

        <div class="typography-row">
          <div class="input-group">
            <label>Font Family</label>
            <div class="theme-picker-wrapper">
              <CustomSelect 
                :model-value="store.fontFamily" 
                :disabled="!licenseStore.isLicensed"
                @change="handleFontFamilyChange" 
                :options="fontFamilies.map(font => ({ value: font.id, label: font.name }))"
              >
                <template #icon>
                  <Type :size="16" style="color: var(--accent);" />
                </template>
              </CustomSelect>
            </div>
          </div>

          <div class="input-group">
            <label>Font Weight</label>
            <CustomSelect 
              :model-value="store.fontWeight" 
              :disabled="!licenseStore.isLicensed"
              @change="handleFontWeightChange" 
              :options="fontWeights.map(weight => ({ value: weight.id, label: weight.name }))"
            />
          </div>

          <div class="input-group">
            <label>Font Style</label>
            <div class="theme-picker-wrapper">
              <CustomSelect 
                :model-value="store.fontStyle" 
                :disabled="!licenseStore.isLicensed"
                @change="handleFontStyleChange" 
                :options="fontStyles.map(style => ({ value: style.id, label: style.name }))"
              >
                <template #icon>
                  <Italic :size="16" style="color: var(--accent);" />
                </template>
              </CustomSelect>
            </div>
          </div>

          <div class="input-group size-group">
            <label>Font Size ({{ store.fontSize }}px)</label>
            <input 
              type="range" 
              min="12" 
              max="20" 
              step="1" 
              :value="store.fontSize" 
              :disabled="!licenseStore.isLicensed"
              @input="handleFontSizeChange" 
              class="font-size-slider"
            />
          </div>
        </div>
      </div>

      <!-- Intelligence Engine -->
      <div class="settings-card">
        <div class="card-header">
          <h3>Intelligence Engine</h3>
          <p>The neural network used for parsing and tailoring.</p>
        </div>
        
        <div class="input-row">
          <div class="input-group">
            <div class="label-row" @mouseenter="activeTooltip = 'provider'" @mouseleave="activeTooltip = null">
              <label>Provider</label>
              <div class="tooltip-trigger">
                <Info :size="12" />
                <AnimatePresence>
                  <Motion
                    v-if="activeTooltip === 'provider'"
                    :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                    :animate="{ opacity: 1, y: 0, scale: 1 }"
                    :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                    :transition="{ duration: 0.15 }"
                    class="flying-message tooltip-top"
                  >
                    Select AI Service
                  </Motion>
                </AnimatePresence>
              </div>
            </div>
            <CustomSelect 
              v-model="providerInput" 
              :options="providers.map(p => ({ value: p.id, label: p.name }))" 
            />
          </div>

          <div class="input-group">
            <div class="label-row" @mouseenter="activeTooltip = 'model'" @mouseleave="activeTooltip = null">
              <label>Active Model</label>
              <div class="tooltip-trigger">
                <Info :size="12" />
                <AnimatePresence>
                  <Motion
                    v-if="activeTooltip === 'model'"
                    :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                    :animate="{ opacity: 1, y: 0, scale: 1 }"
                    :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                    :transition="{ duration: 0.15 }"
                    class="flying-message tooltip-top"
                  >
                    Choose Model Logic
                  </Motion>
                </AnimatePresence>
              </div>
            </div>
            <CustomSelect 
              v-model="modelInput" 
              :options="currentModels.map(m => ({ value: m.id, label: m.name }))" 
            />
          </div>
        </div>

        <!-- Custom Overrides -->
        <div class="input-row overrides-row">
          <div class="input-group">
            <label>Custom Endpoint URL (Optional)</label>
            <input 
              v-model="customBaseUrlInput" 
              type="text" 
              :placeholder="providerInput === 'ollama' ? 'e.g. http://localhost:11434' : 'e.g. https://api.deepseek.com/v1'"
              spellcheck="false"
              class="form-input"
            />
            <span class="setup-tip">
              {{ providerInput === 'ollama' 
                ? 'Override the Ollama service URL (defaults to http://localhost:11434 if blank).' 
                : 'Override the API base URL for this provider (ideal for local or custom OpenAI-compatible endpoints).' }}
            </span>
          </div>

          <div class="input-group">
            <label>Custom Model Name (Optional)</label>
            <input 
              v-model="customModelInput" 
              type="text" 
              placeholder="e.g. deepseek-chat"
              spellcheck="false"
              class="form-input"
            />
            <span class="setup-tip">
              Type a custom model string to override the dropdown selection above.
            </span>
          </div>
        </div>
      </div>

      <!-- API Credentials -->
      <div class="settings-card">
        <div class="card-header">
          <div class="title-row">
            <h3>API Credentials</h3>
            <button 
              v-if="apiKeyInput" 
              class="text-btn" 
              @click="apiKeyInput = ''"
            >Clear Input</button>
          </div>
          <p v-if="providerInput === 'bedrock'">
            AWS Bedrock uses your AWS IAM credentials. Please enter them below to save them securely in your local vault.
          </p>
          <p v-else-if="providerInput === 'ollama'">
            Ollama runs locally and does not require an API key by default. You can leave this blank.
          </p>
          <p v-else>Your {{ providerName }} key is stored in an encrypted vault. It is never sent to our servers.</p>
        </div>
        
        <div class="credentials-content">
          <div class="input-group">
            <label>{{ providerInput === 'bedrock' ? 'AWS Bedrock Credentials' : providerName + ' Secret Key' }}</label>
            <input 
              v-model="apiKeyInput" 
              type="password" 
              :placeholder="providerInput === 'bedrock' 
                ? (store.hasSecureKey ? '•••••••••••••••• (Credentials saved)' : 'access_key_id:secret_access_key:region')
                : providerInput === 'ollama'
                  ? (store.hasSecureKey ? '•••••••••••••••• (Key saved)' : 'Optional (leave blank for local)...')
                  : (store.hasSecureKey ? '•••••••••••••••• (Key saved)' : 'Enter API Key...')"
              spellcheck="false"
              class="form-input"
            />
            <span v-if="providerInput === 'bedrock'" class="setup-tip bedrock-tip">
              Format: <code>ACCESS_KEY_ID:SECRET_ACCESS_KEY:REGION</code>. If region is omitted, it defaults to <code>us-east-1</code>.
            </span>
          </div>

          <div class="credentials-actions">
            <div class="status-area-inline">
              <span v-if="saveError" class="error-msg">
                <span>{{ saveError }}</span>
                <button
                  type="button"
                  class="copy-err-inline-btn"
                  @click="handleCopyInlineError(saveError)"
                  :title="copiedInlineError === saveError ? 'Copied!' : 'Copy Error'"
                >
                  <Check v-if="copiedInlineError === saveError" :size="11" />
                  <Copy v-else :size="11" />
                  <span>{{ copiedInlineError === saveError ? 'Copied!' : 'Copy' }}</span>
                </button>
              </span>
              <transition name="fade">
                <span v-if="showSuccess" class="success-msg">
                  <CheckCircle :size="16" /> Saved
                </span>
              </transition>
            </div>
            
            <div class="button-group">
              <button 
                class="btn-test-connection" 
                @click="handleTestConnection" 
                :disabled="isTestingConnection || isSaving"
              >
                <Play v-if="!isTestingConnection" :size="14" />
                <RotateCcw v-else :size="14" class="spinner" />
                Test Connection
              </button>

              <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'discard'" @mouseleave="activeTooltip = null">
                <button 
                  v-if="hasChanges" 
                  class="btn-action secondary" 
                  @click="syncFromStore" 
                  :disabled="isSaving"
                >
                  <RotateCcw :size="16" />
                </button>
                <AnimatePresence>
                  <Motion
                    v-if="activeTooltip === 'discard' && hasChanges"
                    :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                    :animate="{ opacity: 1, y: 0, scale: 1 }"
                    :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                    :transition="{ duration: 0.15 }"
                    class="flying-message tooltip-bottom-left"
                  >
                    Discard Changes
                  </Motion>
                </AnimatePresence>
              </div>
              
              <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'save'" @mouseleave="activeTooltip = null">
                <button 
                  class="btn-action primary" 
                  @click="handleSave" 
                  :disabled="isSaving || !hasChanges"
                >
                  <Save v-if="!isSaving" :size="16" />
                  <RotateCcw v-else :size="16" class="spinner" />
                </button>
                <AnimatePresence>
                  <Motion
                    v-if="activeTooltip === 'save'"
                    :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                    :animate="{ opacity: 1, y: 0, scale: 1 }"
                    :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                    :transition="{ duration: 0.15 }"
                    class="flying-message tooltip-bottom-left"
                  >
                    Save Configuration
                  </Motion>
                </AnimatePresence>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Backup & Export -->
      <div class="settings-card">
        <div class="card-header">
          <h3>Backup & Export</h3>
          <p>Export your jobs, tailored resumes, and compiler state to a secure JSON file.</p>
        </div>
        
        <div class="export-row">
          <button class="btn-export" @click="exportData" :disabled="isExporting">
            <div class="export-btn-content">
              <Database v-if="!isExporting" :size="18" />
              <RotateCcw v-else :size="18" class="spinner" />
              <div class="export-text">
                <span class="main-text">Generate Full Backup</span>
                <span class="sub-text">Includes all relational data in JSON format</span>
              </div>
            </div>
            <Download :size="18" class="download-icon" />
          </button>
        </div>
      </div>

      <!-- Vault Synchronization -->
      <div class="settings-card">
        <div class="card-header">
          <h3>Vault Synchronization</h3>
          <p>Import data from a backup file to merge with current data or perform a full restore.</p>
        </div>

        <div class="local-backup-picker-card">
          <label class="backup-picker-label">Target Backup File</label>
          <div class="file-picker-row">
            <div class="selected-file-display" :class="{ 'has-file': !!selectedLocalBackupPath }">
              <FileText :size="16" class="file-icon" />
              <span class="file-path-text" :title="selectedLocalBackupPath || 'No backup file selected'">
                {{ selectedLocalBackupPath || 'No backup file selected (Click Browse or click Restore to choose)' }}
              </span>
              <button 
                v-if="selectedLocalBackupPath" 
                type="button" 
                class="btn-clear-file" 
                @click="selectedLocalBackupPath = ''" 
                title="Clear selected file"
              >
                <X :size="14" />
              </button>
            </div>
            <button type="button" class="btn-browse-file" @click="handleBrowseLocalBackup" :disabled="isImporting">
              <FolderOpen :size="15" />
              <span>Browse...</span>
            </button>
          </div>
        </div>
        
        <div class="import-actions">
          <button class="btn-import-option" @click="handleImport('merge')" :disabled="isImporting">
            <RefreshCw :size="18" :class="{ 'spinner': isImporting }" />
            <div class="option-text">
              <span class="option-title">Smart Sync (Merge)</span>
              <span class="option-desc">Add new data without deleting current records</span>
            </div>
          </button>

          <button class="btn-import-option danger" @click="handleImport('overwrite')" :disabled="isImporting">
            <Upload :size="18" />
            <div class="option-text">
              <span class="option-title">Full Restore (Overwrite)</span>
              <span class="option-desc">Replace all current data with the backup file</span>
            </div>
          </button>
        </div>
      </div>

      <!-- Cloud Backup Configuration -->
      <div class="settings-card">
        <div class="card-header">
          <h3>S3 Cloud Backup</h3>
          <p>Configure automated cloud backups to any S3-compatible storage.</p>
        </div>
        
        <div class="input-row">
          <div class="input-group">
            <label>Endpoint URL</label>
            <input type="password" v-model="s3Endpoint" class="form-input" placeholder="e.g. https://s3.us-east-1.amazonaws.com" />
          </div>
          <div class="input-group">
            <label>Bucket Name</label>
            <input type="password" v-model="s3Bucket" class="form-input" placeholder="my-roletect-backup-bucket" />
          </div>
        </div>

        <div class="input-row">
          <div class="input-group">
            <label>Region</label>
            <input type="password" v-model="s3Region" class="form-input" placeholder="us-east-1" />
          </div>
          <div class="input-group">
            <label>Path Style Access</label>
            <label class="checkbox-label">
              <input type="checkbox" v-model="s3ForcePathStyle" class="visually-hidden-checkbox" />
              <div class="custom-checkbox-box" :class="{ checked: s3ForcePathStyle }">
                <Check v-if="s3ForcePathStyle" :size="13" :stroke-width="3" />
              </div>
              <span>Enable (Required for MinIO/R2)</span>
            </label>
          </div>
        </div>
        
        <div class="input-row">
          <div class="input-group">
            <label>Access Key ID</label>
            <input type="password" v-model="s3AccessKey" class="form-input" :placeholder="hasS3AccessKey ? '•••••••••••••••• (Saved)' : 'Enter AWS/S3 Access Key ID'" />
          </div>
          <div class="input-group">
            <label>Secret Access Key</label>
            <input type="password" v-model="s3SecretKey" class="form-input" :placeholder="hasS3SecretKey ? '•••••••••••••••• (Saved)' : 'Enter AWS/S3 Secret Access Key'" />
          </div>
        </div>
        
        <div class="credentials-actions credentials-actions-s3">
          <div class="button-group s3-button-group">
            <button class="btn-test-connection" @click="handleTestS3(false)" :disabled="isTestingS3 || isSavingS3">
              <RefreshCw v-if="isTestingS3" :size="14" class="spinner" />
              <Play v-else :size="14" />
              Test Connection
            </button>
            <button class="btn-action primary btn-save-s3" @click="handleSaveS3" :disabled="isSavingS3">
              <Save :size="14" />
              Save S3 Settings
            </button>
          </div>
          <div class="status-area-inline s3-status-area">
            <span v-if="s3TestSuccess" class="success-msg"><CheckCircle :size="14"/> {{ isSavingS3 ? '' : 'Settings saved.' }}</span>
            <span v-if="s3TestError" class="error-msg">
              <span>{{ s3TestError }}</span>
              <button
                type="button"
                class="copy-err-inline-btn"
                @click="handleCopyInlineError(s3TestError)"
                :title="copiedInlineError === s3TestError ? 'Copied!' : 'Copy Error'"
              >
                <Check v-if="copiedInlineError === s3TestError" :size="11" />
                <Copy v-else :size="11" />
                <span>{{ copiedInlineError === s3TestError ? 'Copied!' : 'Copy' }}</span>
              </button>
            </span>
            <span v-if="!s3TestError && !s3TestSuccess && s3SetupOk" class="s3-auto-status">
              <strong class="s3-auto-strong">Auto-Backup: Active</strong>
              <span>Last Upload: {{ s3LastUpload }}</span>
            </span>
          </div>
        </div>
      </div>

      <!-- Cloud Restore Configuration -->
      <div class="settings-card">
        <div class="card-header">
          <h3>Restore from Cloud</h3>
          <p>Fetch and restore your data from your S3 backup vault. <strong>Warning:</strong> Restoring will modify local data.</p>
        </div>
        
        <div class="credentials-actions restore-fetch-actions">
          <div class="button-group restore-button-group">
            <button class="btn-test-connection btn-fetch-backups" @click="handleFetchBackups" :disabled="isFetchingBackups || !s3SetupOk">
              <DownloadCloud v-if="!isFetchingBackups" :size="14" />
              <RefreshCw v-else :size="14" class="spinner" />
              Fetch Available Backups
            </button>
          </div>
          <div class="status-message">
            <span v-if="fetchBackupsError" class="error-msg">
              <span>{{ fetchBackupsError }}</span>
              <button
                type="button"
                class="copy-err-inline-btn"
                @click="handleCopyInlineError(fetchBackupsError)"
                :title="copiedInlineError === fetchBackupsError ? 'Copied!' : 'Copy Error'"
              >
                <Check v-if="copiedInlineError === fetchBackupsError" :size="11" />
                <Copy v-else :size="11" />
                <span>{{ copiedInlineError === fetchBackupsError ? 'Copied!' : 'Copy' }}</span>
              </button>
            </span>
            <span v-if="!s3SetupOk" class="warning-msg">S3 not active.</span>
          </div>
        </div>

        <div v-if="availableBackups.length > 0" class="input-row restore-inputs">
          <div class="input-group">
            <label>Select Backup to Restore</label>
            <CustomSelect
              v-model="selectedBackup"
              :options="availableBackupsOptions"
              class="custom-select"
              placement="top"
            />
          </div>
          <div class="input-group">
            <label>Restore Mode</label>
            <CustomSelect
              v-model="restoreMode"
              :options="restoreModeOptions"
              class="custom-select"
              placement="top"
            />
          </div>
        </div>
        
        <div v-if="availableBackups.length > 0" class="credentials-actions restore-submit-actions">
          <button class="btn-restore-submit" @click="handleRestoreBackup" :disabled="isRestoringBackup || !selectedBackup">
            <RotateCcw v-if="!isRestoringBackup" :size="16" />
            <RefreshCw v-else :size="16" class="spinner" />
            {{ isRestoringBackup ? 'Restoring & Reloading...' : 'Restore Selected Backup' }}
          </button>
        </div>
      </div>

      <!-- Backup Automation Configuration -->
      <div class="settings-card">
        <div class="card-header">
          <h3>Backup Automation</h3>
          <p>Control what happens automatically when you close RoleTect.</p>
        </div>
        
        <div class="input-row backup-automation-row">
          <div class="input-group">
            <label class="checkbox-label">
              <input type="checkbox" v-model="autoLocalBackup" class="visually-hidden-checkbox" />
              <div class="custom-checkbox-box" :class="{ checked: autoLocalBackup }">
                <Check v-if="autoLocalBackup" :size="13" :stroke-width="3" />
              </div>
              <span>Auto Local Backup on Exit</span>
            </label>
            <p class="backup-tip">Automatically exports an unencrypted snapshot to your <strong>Documents/RoleTect-Backups</strong> folder.</p>
          </div>
        </div>

        <div class="input-row backup-automation-row">
          <div class="input-group">
            <label class="checkbox-label" :class="{ disabled: !s3SetupOk }">
              <input type="checkbox" v-model="autoCloudBackup" class="visually-hidden-checkbox" :disabled="!s3SetupOk" />
              <div class="custom-checkbox-box" :class="{ checked: autoCloudBackup, disabled: !s3SetupOk }">
                <Check v-if="autoCloudBackup" :size="13" :stroke-width="3" />
              </div>
              <span>Auto Cloud Backup on Exit</span>
            </label>
            <p class="backup-tip">Automatically uploads an encrypted snapshot to your configured S3 bucket. Requires active S3 setup.</p>
          </div>
        </div>
      </div>

      <!-- Maintenance -->
      <div class="settings-card">
        <div class="card-header">
          <h3>Maintenance</h3>
          <p>Advanced tools to repair the engine or resolve environmental issues.</p>
        </div>
        
        <div class="maintenance-row">
          <button class="btn-maintenance" @click="handleClearCache" :disabled="isClearingCache">
            <div class="maintenance-btn-content">
              <RotateCcw v-if="!isClearingCache" :size="18" />
              <RefreshCw v-else :size="18" class="spinner" />
              <div class="maintenance-text">
                <span class="main-text">Purge LaTeX Cache</span>
                <span class="sub-text">Resolves "fatal format file error" by forcing a fresh engine rebuild</span>
              </div>
            </div>
          </button>
        </div>
      </div>

      <!-- Diagnostics & Error Audit Trail -->
      <div class="settings-card error-audit-card">
        <div class="card-header">
          <div class="title-row">
            <div class="title-with-badge">
              <AlertTriangle :size="16" class="warning-icon" />
              <h3>Diagnostics & Error Audit Trail</h3>
            </div>
          </div>
          <p>Time-by-time error ledger capturing creating, compiling, fetching, and AI tasks with diagnostic filters and 1-click copying.</p>
        </div>

        <div class="error-audit-card-body">
          <ErrorAuditViewer />
        </div>
      </div>

      <!-- License & Activation -->
      <div class="settings-card" id="license-activation">
        <div class="card-header">
          <div class="title-row">
            <h3>License & Subscription</h3>
            <div class="header-btns">
              <span v-if="licenseStore.isLicensed" class="badge-status-active">
                <ShieldCheck :size="14" />
                {{ licenseStore.licenseStatus?.trial ? 'Trial Active' : 'Licensed' }}
              </span>
              <span v-else class="badge-status-free">
                <Lock :size="12" />
                Free Tier
              </span>
            </div>
          </div>
          <p>Manage your Lemon Squeezy license activation and device authorization.</p>
        </div>

        <div class="license-info-section">
          <div class="info-row">
            <div v-if="licenseStore.licenseStatus?.customer_email" class="license-detail-item">
              <span class="detail-label">Registered To</span>
              <span class="detail-val">{{ licenseStore.licenseStatus.customer_email }}</span>
            </div>

            <div v-if="licenseStore.licenseStatus?.trial_ends_at" class="license-detail-item">
              <span class="detail-label">Trial Expiration</span>
              <span class="detail-val trial-val">{{ new Date(licenseStore.licenseStatus.trial_ends_at).toLocaleDateString() }}</span>
            </div>

            <div v-if="licenseStore.licenseStatus?.license_key" class="license-detail-item">
              <span class="detail-label">License Key</span>
              <span class="detail-val key-val">
                ••••••••-••••-{{ licenseStore.licenseStatus.license_key.slice(-8) }}
              </span>
            </div>

            <!-- Upgrade / Enter License Key -->
            <div v-if="!licenseStore.licenseStatus?.license_key" class="license-upgrade-box">
              <div class="upgrade-header-row">
                <div class="upgrade-header-left">
                  <Sparkles :size="14" style="color: var(--accent);" />
                  <span class="upgrade-title">Have a License Key?</span>
                </div>
                <button
                  type="button"
                  class="btn-get-license"
                  @click="openUrl('https://github.com/AhmedTrooper/roletect-app')"
                >
                  <ExternalLink :size="12" />
                  <span>Get License</span>
                </button>
              </div>
              <p class="upgrade-desc">
                Activate your permanent license key from Lemon Squeezy to upgrade to RoleTect Pro.
              </p>
              <div class="upgrade-input-row">
                <div class="upgrade-input-inner">
                  <input
                    id="upgrade-key-input"
                    v-model="upgradeKeyInput"
                    type="text"
                    placeholder="XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX"
                    class="upgrade-key-field"
                    :disabled="isActivatingUpgrade"
                  />
                  <button
                    type="button"
                    class="btn-paste-key"
                    @click="handlePasteUpgradeKey"
                    :disabled="isActivatingUpgrade"
                    title="Paste from clipboard"
                  >
                    <ClipboardPaste :size="14" />
                  </button>
                </div>
                <button
                  type="button"
                  class="btn-activate-key"
                  @click="handleActivateUpgrade"
                  :disabled="isActivatingUpgrade || !upgradeKeyInput.trim()"
                >
                  <ShieldCheck :size="14" />
                  <span>{{ isActivatingUpgrade ? 'Activating...' : 'Activate' }}</span>
                </button>
              </div>
            </div>
          </div>

          <div v-if="licenseStore.licenseStatus?.license_key" class="license-actions">
            <button 
              type="button" 
              class="license-btn verify-btn" 
              @click="handleSyncLicenseSettings"
              :disabled="isSyncingLicenseSettings"
            >
              <RefreshCw :size="14" :class="{ 'spinner': isSyncingLicenseSettings }" />
              <span>{{ isSyncingLicenseSettings ? 'Verifying Online...' : 'Verify Status with Lemon Squeezy' }}</span>
            </button>

            <button
              v-if="licenseStore.isLicensed"
              type="button"
              class="license-btn cancel-btn"
              @click="handleCancelSubscription"
            >
              <LogOut :size="14" />
              <span>Cancel Subscription</span>
            </button>

            <button 
              type="button" 
              class="license-btn deactivate-btn" 
              @click="handleDeactivateLicense"
              :disabled="isDeactivatingLicense"
            >
              <LogOut :size="14" />
              <span>{{ isDeactivatingLicense ? 'Deactivating...' : 'Deactivate License on This Device' }}</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* =======================================================================
   Base Container & Header
   ======================================================================= */
.settings-container {
  padding: 40px 32px 100px 32px;
  max-width: 900px;
  width: 100%;
  margin: 0 auto;
  box-sizing: border-box;
  min-width: 0;
}

.header {
  margin-bottom: 32px;
  width: 100%;
}

.header h2 {
  font-size: clamp(1.4rem, 4vw, 2rem);
  font-weight: 700;
  margin: 0;
  color: var(--ink);
  line-height: 1.25;
}

.subtitle {
  color: var(--muted);
  margin: 8px 0 0;
  font-size: clamp(0.78rem, 2vw, 0.9rem);
  line-height: 1.4;
  word-break: break-word;
}

.settings-grid {
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding-bottom: 60px;
  width: 100%;
  min-width: 0;
}

/* =======================================================================
   Card Component Base
   ======================================================================= */
.settings-card {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 16px;
  padding: 24px;
  box-shadow: var(--shadow);
  width: 100%;
  box-sizing: border-box;
  min-width: 0;
}

.card-header {
  width: 100%;
  min-width: 0;
}

.card-header h3 {
  font-size: 1.05rem;
  font-weight: 700;
  margin: 0;
  color: var(--ink);
}

.card-header p {
  color: var(--muted);
  font-size: 0.82rem;
  margin: 6px 0 0;
  line-height: 1.45;
  word-break: break-word;
}

.title-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  width: 100%;
}

.title-with-badge {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  min-width: 0;
}

.capped-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  font-size: 0.68rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  border-radius: 12px;
  background: rgba(234, 179, 8, 0.12);
  border: 1px solid rgba(234, 179, 8, 0.35);
  color: #eab308;
  flex-shrink: 0;
}

.header-btns {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}

.text-btn {
  background: none;
  border: none;
  color: var(--accent);
  font-weight: 700;
  font-size: 0.75rem;
  text-transform: uppercase;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border-radius: var(--radius-sm, 4px);
  transition: all 0.15s ease;
  min-height: 32px;
  white-space: nowrap;
}

.text-btn.secondary {
  color: var(--muted);
}

.text-btn.secondary:hover {
  color: var(--ink);
}

.text-btn:hover {
  opacity: 0.85;
}

.text-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

/* Feature Capped Banner */
.feature-capped-banner {
  margin: 12px 0 16px 0;
  padding: 12px 14px;
  background: rgba(234, 179, 8, 0.06);
  border: 1px dashed rgba(234, 179, 8, 0.3);
  border-radius: 8px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  box-sizing: border-box;
  width: 100%;
}

.banner-content {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  min-width: 0;
}

.banner-icon {
  color: #eab308;
  flex-shrink: 0;
  margin-top: 2px;
}

.banner-text {
  min-width: 0;
}

.banner-text strong {
  display: block;
  font-size: 0.82rem;
  color: var(--ink);
  margin-bottom: 2px;
}

.banner-text p {
  font-size: 0.78rem;
  color: var(--muted);
  margin: 0;
  line-height: 1.4;
  word-break: break-word;
}

.btn-activate-inline {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: 6px;
  font-size: 0.78rem;
  font-weight: 600;
  white-space: nowrap;
  cursor: pointer;
  transition: opacity 0.15s ease, transform 0.15s ease;
  flex-shrink: 0;
  min-height: 38px;
}

.btn-activate-inline:hover {
  opacity: 0.9;
  transform: translateY(-1px);
}

/* =======================================================================
   Theme Selector & Import
   ======================================================================= */
.theme-selector-row {
  display: flex;
  align-items: flex-end;
  gap: 12px;
  margin-top: 20px;
  width: 100%;
}

.theme-picker-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  width: 100%;
  min-width: 0;
}

.delete-theme-btn {
  height: 42px;
  width: 42px;
  min-width: 42px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(248, 81, 73, 0.1);
  border: 1px solid rgba(248, 81, 73, 0.2);
  color: var(--warning);
  border-radius: 8px;
  cursor: pointer;
  transition: 0.2s;
  flex-shrink: 0;
}

.delete-theme-btn:hover:not(:disabled) {
  background: var(--warning);
  color: white;
  border-color: var(--warning);
}

.delete-theme-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.import-theme-area {
  margin-top: 24px;
  padding-top: 24px;
  border-top: 1px solid var(--line);
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 100%;
  box-sizing: border-box;
}

.import-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
}

.help-link-btn {
  background: none;
  border: none;
  color: var(--accent);
  font-size: 0.7rem;
  font-weight: 700;
  text-transform: uppercase;
  cursor: pointer;
  padding: 4px 6px;
  border-radius: 4px;
}

.theme-textarea {
  width: 100%;
  height: 120px;
  background: var(--bg);
  border: 1px solid var(--line);
  border-radius: 8px;
  padding: 12px;
  color: var(--ink);
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.8rem;
  resize: vertical;
  box-sizing: border-box;
  scrollbar-width: thin;
  scrollbar-color: var(--line) transparent;
}

.theme-textarea::-webkit-scrollbar {
  width: 5px;
}

.theme-textarea::-webkit-scrollbar-track {
  background: transparent;
  margin: 6px 0;
}

.theme-textarea::-webkit-scrollbar-thumb {
  background: var(--line);
  border-radius: 4px;
}

.theme-textarea::-webkit-scrollbar-thumb:hover {
  background: var(--accent);
}

.import-actions-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.btn-import-confirm {
  background: var(--accent);
  color: white;
  border: none;
  border-radius: 6px;
  padding: 8px 16px;
  font-size: 0.8rem;
  font-weight: 700;
  cursor: pointer;
  min-height: 38px;
  transition: opacity 0.15s ease;
}

.btn-import-confirm:hover {
  opacity: 0.9;
}

.error-inline {
  color: var(--warning);
  font-size: 0.75rem;
  font-weight: 600;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  word-break: break-word;
}

/* =======================================================================
   Typography Settings
   ======================================================================= */
.typography-row {
  display: flex;
  gap: 24px;
  margin-top: 20px;
  align-items: flex-end;
  width: 100%;
}

.size-group {
  flex: 1.5;
}

.font-size-slider {
  width: 100%;
  height: 32px;
  margin-top: 8px;
  cursor: pointer;
  accent-color: var(--accent);
}

.font-size-slider:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* =======================================================================
   Input Rows & Controls
   ======================================================================= */
.input-row {
  display: flex;
  gap: 20px;
  margin-top: 20px;
  width: 100%;
}

.input-group {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
  width: 100%;
}

.label-row {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: help;
}

.tooltip-trigger {
  color: var(--muted);
  display: flex;
  align-items: center;
  position: relative;
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
  box-shadow: 0 4px 12px rgba(0,0,0,0.3);
  border: 1px solid var(--line);
}

.tooltip-top {
  bottom: calc(100% + 6px);
  left: 50%;
  transform: translateX(-50%);
}

.tooltip-bottom-left {
  top: calc(100% + 6px);
  right: 0;
}

.btn-tooltip-wrapper {
  position: relative;
  display: flex;
}

label {
  color: var(--accent);
  font-weight: 700;
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
}

.form-input, .custom-select {
  width: 100%;
  padding: 10px 14px;
  min-height: 42px;
  font-size: 0.95rem;
  background: var(--surface-soft);
  border: 1px solid var(--line);
  color: var(--ink);
  border-radius: 8px;
  outline: none;
  box-sizing: border-box;
  transition: border-color 0.2s, box-shadow 0.2s;
}

.form-input:focus, .custom-select:focus {
  border-color: var(--accent);
}

.setup-tip {
  font-size: 0.8rem;
  color: var(--muted);
  margin-top: 4px;
  display: block;
  line-height: 1.4;
  word-break: break-word;
}

.bedrock-tip {
  margin-top: 8px;
  font-size: 0.85rem;
}

/* =======================================================================
   Credentials Actions & Buttons
   ======================================================================= */
.credentials-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
  margin-top: 20px;
  width: 100%;
}

.credentials-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 16px;
  border-top: 1px solid var(--line);
  gap: 16px;
  flex-wrap: wrap;
  width: 100%;
  box-sizing: border-box;
}

.status-area-inline {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
  min-width: 0;
}

.button-group {
  display: flex;
  gap: 12px;
  align-items: center;
  flex-shrink: 0;
}

.btn-test-connection {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  background: var(--bg-accent);
  border: 1px solid var(--line);
  color: var(--accent);
  font-weight: 700;
  font-size: 0.75rem;
  text-transform: uppercase;
  border-radius: 10px;
  padding: 0 16px;
  height: 42px;
  min-height: 42px;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.btn-test-connection:hover:not(:disabled) {
  border-color: var(--accent);
  background: var(--surface-soft);
}

.btn-test-connection:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-action {
  width: 42px;
  height: 42px;
  min-width: 42px;
  min-height: 42px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 10px;
  cursor: pointer;
  transition: 0.2s;
  border: 1px solid var(--line);
  flex-shrink: 0;
}

.btn-action.primary {
  background: var(--accent);
  color: white;
  border-color: var(--accent);
}

.btn-action.primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-action.secondary {
  background: none;
  color: var(--muted);
}

.btn-action.secondary:hover {
  border-color: var(--ink);
  color: var(--ink);
}

.btn-save-s3 {
  width: auto;
  padding: 0 16px;
  font-weight: 700;
  gap: 8px;
  font-size: 0.75rem;
  text-transform: uppercase;
  white-space: nowrap;
}

.s3-auto-status {
  font-size: 0.75rem;
  color: var(--muted);
  display: flex;
  flex-direction: column;
  text-align: right;
}

.s3-auto-strong {
  color: var(--accent);
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  color: var(--ink);
  font-size: 0.88rem;
  font-weight: 500;
  text-transform: none;
  letter-spacing: normal;
  min-height: 38px;
  user-select: none;
}

.checkbox-label.disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.visually-hidden-checkbox {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

.custom-checkbox-box {
  width: 20px;
  height: 20px;
  border-radius: 5px;
  border: 1.5px solid var(--border);
  background: var(--bg-card);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: #ffffff;
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  flex-shrink: 0;
  box-sizing: border-box;
}

.custom-checkbox-box.checked {
  background: var(--accent);
  border-color: var(--accent);
}

.checkbox-label:hover .custom-checkbox-box:not(.disabled) {
  border-color: var(--accent);
}

.custom-checkbox-box.disabled {
  opacity: 0.55;
  cursor: not-allowed;
  border-color: var(--border);
}

/* Local backup file picker */
.local-backup-picker-card {
  margin-bottom: 16px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.backup-picker-label {
  font-size: 0.82rem;
  font-weight: 600;
  color: var(--muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.file-picker-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.selected-file-display {
  flex: 1;
  min-height: 38px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 0 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  overflow: hidden;
  font-size: 0.85rem;
  color: var(--muted);
  transition: border-color 0.2s;
  box-sizing: border-box;
}

.selected-file-display.has-file {
  color: var(--text-primary);
  border-color: var(--accent);
}

.selected-file-display .file-icon {
  flex-shrink: 0;
  color: var(--accent);
}

.selected-file-display .file-path-text {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  direction: rtl;
  text-align: left;
}

.btn-clear-file {
  background: transparent;
  border: none;
  color: var(--muted);
  cursor: pointer;
  padding: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: color 0.15s, background-color 0.15s;
}

.btn-clear-file:hover {
  color: var(--error);
  background: rgba(239, 68, 68, 0.1);
}

.btn-browse-file {
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-primary);
  padding: 0 14px;
  height: 38px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.85rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
  flex-shrink: 0;
}

.btn-browse-file:hover:not(:disabled) {
  border-color: var(--accent);
  color: var(--accent);
  background: var(--bg-card);
}

.backup-tip {
  font-size: 0.75rem;
  color: var(--muted);
  margin-top: 4px;
  margin-left: 30px;
  line-height: 1.4;
  word-break: break-word;
}

/* =======================================================================
   Cloud Restore
   ======================================================================= */
.btn-restore-submit {
  width: 100%;
  min-height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--warning);
  color: white;
  border: none;
  border-radius: 10px;
  font-weight: 700;
  font-size: 0.85rem;
  gap: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-restore-submit:hover:not(:disabled) {
  opacity: 0.92;
  transform: translateY(-1px);
}

.btn-restore-submit:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* =======================================================================
   Backup & Vault Sync
   ======================================================================= */
.export-row {
  margin-top: 24px;
  width: 100%;
}

.btn-export {
  width: 100%;
  min-height: 52px;
  background: var(--surface-soft);
  border: 1px solid var(--line);
  border-radius: 12px;
  padding: 14px 18px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  color: var(--ink);
  gap: 12px;
  box-sizing: border-box;
}

.btn-export:hover:not(:disabled) {
  background: var(--surface);
  border-color: var(--accent);
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
}

.btn-export:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.export-btn-content {
  display: flex;
  align-items: center;
  gap: 14px;
  text-align: left;
  min-width: 0;
}

.export-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.main-text {
  font-size: 0.9rem;
  font-weight: 700;
  color: var(--ink);
  word-break: break-word;
}

.sub-text {
  font-size: 0.7rem;
  color: var(--muted);
  word-break: break-word;
}

.download-icon {
  color: var(--accent);
  opacity: 0.8;
  flex-shrink: 0;
}

.btn-export:hover .download-icon {
  opacity: 1;
  transform: translateY(2px);
}

.import-actions {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  margin-top: 24px;
  width: 100%;
}

.btn-import-option {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 16px;
  min-height: 52px;
  background: var(--surface-soft);
  border: 1px solid var(--line);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
  text-align: left;
  color: var(--ink);
  box-sizing: border-box;
  min-width: 0;
}

.btn-import-option:hover:not(:disabled) {
  border-color: var(--accent);
  background: var(--surface);
  transform: translateY(-2px);
}

.btn-import-option.danger:hover:not(:disabled) {
  border-color: var(--warning);
  background: rgba(248, 81, 73, 0.05);
}

.option-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.option-title {
  font-size: 0.85rem;
  font-weight: 700;
  word-break: break-word;
}

.option-desc {
  font-size: 0.65rem;
  color: var(--muted);
  word-break: break-word;
}

/* =======================================================================
   Maintenance
   ======================================================================= */
.maintenance-row {
  margin-top: 24px;
  width: 100%;
}

.btn-maintenance {
  width: 100%;
  min-height: 52px;
  background: rgba(248, 81, 73, 0.05);
  border: 1px solid rgba(248, 81, 73, 0.1);
  border-radius: 12px;
  padding: 14px 18px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  cursor: pointer;
  transition: all 0.2s ease;
  color: var(--warning);
  gap: 12px;
  box-sizing: border-box;
}

.btn-maintenance:hover:not(:disabled) {
  background: rgba(248, 81, 73, 0.1);
  border-color: var(--warning);
  transform: translateY(-2px);
}

.maintenance-btn-content {
  display: flex;
  align-items: center;
  gap: 14px;
  text-align: left;
  min-width: 0;
}

.maintenance-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

/* =======================================================================
   Diagnostics & Error Audit Card
   ======================================================================= */
.error-audit-card {
  overflow: visible;
}

.error-audit-card-body {
  margin-top: 14px;
  width: 100%;
  min-width: 0;
}

.warning-icon {
  color: var(--warning);
  flex-shrink: 0;
}

/* =======================================================================
   License & Subscription Section
   ======================================================================= */
.license-info-section {
  margin-top: 16px;
  width: 100%;
  min-width: 0;
}

.info-row {
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 100%;
}

.license-detail-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  background: var(--bg);
  border: 1px solid var(--line);
  border-radius: 8px;
  font-size: 0.85rem;
  width: 100%;
  box-sizing: border-box;
  min-width: 0;
  gap: 8px;
}

.detail-label {
  color: var(--muted);
  flex-shrink: 0;
}

.detail-val {
  color: var(--ink);
  font-weight: 500;
  word-break: break-all;
  text-align: right;
}

.trial-val {
  color: var(--accent);
}

.key-val {
  font-family: monospace;
  font-size: 0.8rem;
}

.license-upgrade-box {
  margin-top: 14px;
  padding: 14px;
  background: var(--bg);
  border: 1px dashed var(--line);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 100%;
  box-sizing: border-box;
}

.upgrade-header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.upgrade-header-left {
  display: flex;
  align-items: center;
  gap: 6px;
}

.upgrade-title {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--ink);
}

.btn-get-license {
  background: transparent;
  border: none;
  color: var(--accent);
  font-size: 0.8rem;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  text-decoration: underline;
  padding: 2px 4px;
}

.upgrade-desc {
  font-size: 0.8rem;
  color: var(--muted);
  margin: 0;
  line-height: 1.4;
  word-break: break-word;
}

.upgrade-input-row {
  display: flex;
  gap: 8px;
  width: 100%;
}

.upgrade-input-inner {
  display: flex;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.upgrade-key-field {
  flex: 1;
  min-width: 0;
  padding: 8px 12px;
  font-family: monospace;
  font-size: 0.8rem;
  background: var(--surface-soft);
  border: 1px solid var(--line);
  border-radius: 6px;
  color: var(--ink);
  min-height: 40px;
  box-sizing: border-box;
}

.btn-paste-key {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 8px 10px;
  background: var(--surface-soft);
  border: 1px solid var(--line);
  color: var(--muted);
  border-radius: 6px;
  font-size: 0.8rem;
  cursor: pointer;
  min-height: 40px;
  min-width: 40px;
  flex-shrink: 0;
  transition: all 0.15s ease;
}

.btn-paste-key:hover:not(:disabled) {
  color: var(--ink);
  border-color: var(--accent);
}

.btn-activate-key {
  padding: 8px 16px;
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: 6px;
  font-size: 0.8rem;
  font-weight: 600;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  min-height: 40px;
  white-space: nowrap;
  flex-shrink: 0;
  transition: opacity 0.15s ease;
}

.btn-activate-key:hover:not(:disabled) {
  opacity: 0.9;
}

.btn-activate-key:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.license-actions {
  margin-top: 18px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
  width: 100%;
}

.license-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px 16px;
  border-radius: 8px;
  font-size: 0.85rem;
  cursor: pointer;
  min-height: 42px;
  font-weight: 600;
  transition: all 0.15s ease;
  white-space: nowrap;
}

.license-btn.verify-btn {
  background: var(--surface-soft);
  border: 1px solid var(--line);
  color: var(--ink);
}

.license-btn.verify-btn:hover:not(:disabled) {
  border-color: var(--accent);
}

.license-btn.cancel-btn {
  background: rgba(250, 189, 47, 0.12);
  border: 1px solid rgba(250, 189, 47, 0.4);
  color: #fabd2f;
}

.license-btn.cancel-btn:hover {
  background: rgba(250, 189, 47, 0.2);
}

.license-btn.deactivate-btn {
  background: rgba(248, 81, 73, 0.1);
  border: 1px solid rgba(248, 81, 73, 0.3);
  color: var(--warning);
}

.license-btn.deactivate-btn:hover:not(:disabled) {
  background: rgba(248, 81, 73, 0.2);
}

.license-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.badge-status-active {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  background: var(--accent-soft, rgba(35, 134, 54, 0.15));
  border: 1px solid var(--accent, #238636);
  border-radius: 20px;
  color: var(--accent, #238636);
  font-size: 0.75rem;
  font-weight: 600;
  flex-shrink: 0;
}

.badge-status-free {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--line, rgba(255, 255, 255, 0.1));
  border-radius: 20px;
  color: var(--muted);
  font-size: 0.75rem;
  font-weight: 600;
  flex-shrink: 0;
}

.copy-err-inline-btn {
  background: transparent;
  border: 1px solid var(--line);
  color: var(--muted);
  border-radius: 4px;
  padding: 2px 6px;
  font-size: 0.7rem;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  margin-left: 6px;
  transition: all 0.15s ease;
}

.copy-err-inline-btn:hover {
  color: var(--ink);
  border-color: var(--accent);
}

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.success-msg {
  color: var(--accent);
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.8rem;
}

.error-msg {
  color: var(--warning);
  font-weight: 600;
  font-size: 0.8rem;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  word-break: break-word;
  max-width: 100%;
}

.warning-msg {
  color: var(--warning);
  font-size: 0.85rem;
}

@keyframes inputPulse {
  0% { box-shadow: 0 0 0 0 rgba(35, 134, 54, 0.7); }
  50% { box-shadow: 0 0 0 6px rgba(35, 134, 54, 0.25); }
  100% { box-shadow: 0 0 0 0 rgba(35, 134, 54, 0); }
}

.pulse-focus {
  animation: inputPulse 0.9s ease-out 2;
  border-color: var(--accent) !important;
  outline: none;
}

/* =======================================================================
   Tablet Responsive Tier (601px - 959px)
   ======================================================================= */
@media (max-width: 959px) and (min-width: 601px) {
  .settings-container {
    padding: 28px 20px 90px 20px;
  }
  .typography-row {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 16px;
    align-items: flex-end;
  }
}

/* =======================================================================
   Mobile Responsive Tier (<= 600px)
   ======================================================================= */
@media (max-width: 600px) {
  .settings-container {
    padding: 16px 12px 90px 12px;
  }
  .header {
    margin-bottom: 20px;
  }
  .settings-grid {
    gap: 16px;
    padding-bottom: 30px;
  }
  .settings-card {
    padding: 16px 12px;
    border-radius: 12px;
  }
  .title-row {
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }
  .header-btns {
    flex-wrap: wrap;
    gap: 8px;
    width: 100%;
  }
  .text-btn {
    min-height: 38px;
    padding: 6px 10px;
    background: var(--surface-soft);
    border: 1px solid var(--line);
    border-radius: 6px;
  }
  .feature-capped-banner {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
  }
  .btn-activate-inline {
    width: 100%;
    justify-content: center;
    min-height: 40px;
  }
  .typography-row {
    flex-direction: column;
    align-items: stretch;
    gap: 14px;
  }
  .input-row {
    flex-direction: column;
    gap: 14px;
    margin-top: 14px;
  }
  .file-picker-row {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }
  .btn-browse-file {
    width: 100%;
    justify-content: center;
  }
  .import-actions {
    grid-template-columns: 1fr;
    gap: 12px;
    margin-top: 16px;
  }
  .credentials-actions {
    flex-direction: column;
    align-items: stretch;
    gap: 14px;
  }
  .button-group {
    flex-wrap: wrap;
    width: 100%;
    gap: 10px;
  }
  .btn-test-connection {
    flex: 1 1 auto;
    min-width: 130px;
    justify-content: center;
  }
  .s3-button-group {
    flex-direction: column;
    width: 100%;
  }
  .s3-button-group .btn-test-connection,
  .s3-button-group .btn-save-s3 {
    width: 100%;
    justify-content: center;
    min-height: 42px;
  }
  .s3-status-area {
    width: 100%;
    align-items: flex-start;
    text-align: left;
  }
  .s3-auto-status {
    text-align: left !important;
  }
  .backup-tip {
    margin-left: 0;
    margin-top: 2px;
  }
  .license-detail-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }
  .detail-val {
    text-align: left;
  }
  .upgrade-input-row {
    flex-direction: column;
    gap: 10px;
  }
  .btn-activate-key {
    width: 100%;
    min-height: 42px;
  }
  .license-actions {
    flex-direction: column;
    align-items: stretch;
    gap: 10px;
  }
  .license-btn {
    width: 100%;
  }
}

/* =======================================================================
   Ultra-Compact Mobile Tier (<= 340px)
   ======================================================================= */
@media (max-width: 340px) {
  .settings-container {
    padding: 10px 6px 80px 6px;
  }
  .settings-card {
    padding: 12px 8px;
    border-radius: 10px;
  }
  .header h2 {
    font-size: 1.25rem;
  }
  .btn-export, .btn-import-option, .btn-maintenance {
    padding: 12px 10px;
  }
  .export-btn-content, .maintenance-btn-content {
    gap: 10px;
  }
  .btn-action {
    width: 38px;
    height: 38px;
    min-width: 38px;
    min-height: 38px;
  }
  .upgrade-input-inner {
    flex-direction: column;
  }
  .btn-paste-key {
    width: 100%;
  }
}
</style>
