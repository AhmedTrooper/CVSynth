<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue';
import { useSettingsStore } from '../store/settings';

const store = useSettingsStore();

const providerInput = ref('openai');
const modelInput = ref('gpt-4o');
const apiKeyInput = ref('');
const isSaving = ref(false);
const showSuccess = ref(false);

// Dynamic Configuration
const providers = [
  { id: 'openai', name: 'OpenAI' },
  { id: 'gemini', name: 'Google Gemini' },
  { id: 'groq', name: 'Groq (Ultra-Fast)' }
];

const modelsByProvider: Record<string, {id: string, name: string}[]> = {
  openai: [
    { id: 'gpt-4o', name: 'GPT-4o (Best Logic)' },
    { id: 'gpt-4-turbo', name: 'GPT-4 Turbo' },
    { id: 'gpt-3.5-turbo', name: 'GPT-3.5 Fast' }
  ],
  gemini: [
    { id: 'gemini-1.5-pro', name: 'Gemini 1.5 Pro' },
    { id: 'gemini-1.5-flash', name: 'Gemini 1.5 Flash' }
  ],
  groq: [
    { id: 'llama3-70b-8192', name: 'Llama 3 70B' },
    { id: 'mixtral-8x7b-32768', name: 'Mixtral 8x7B' },
    { id: 'llama3-8b-8192', name: 'Llama 3 8B' }
  ]
};

// Auto-switch to the first model if the provider changes
watch(providerInput, (newProvider) => {
  const availableModels = modelsByProvider[newProvider];
  if (!availableModels.find(m => m.id === modelInput.value)) {
    modelInput.value = availableModels[0].id;
  }
});

const currentModels = computed(() => modelsByProvider[providerInput.value] || []);
const providerName = computed(() => providers.find(p => p.id === providerInput.value)?.name || 'AI');

onMounted(async () => {
  await store.loadSettings();
  providerInput.value = store.selectedAiProvider;
  modelInput.value = store.selectedAiModel;
});

const handleSave = async () => {
  isSaving.value = true;
  showSuccess.value = false;
  
  try {
    if (apiKeyInput.value.trim() !== '') {
      await store.saveApiKey(apiKeyInput.value.trim());
      apiKeyInput.value = ''; 
    }
    await store.saveModelConfig(providerInput.value, modelInput.value);
    
    showSuccess.value = true;
    setTimeout(() => { showSuccess.value = false; }, 3000);
  } catch (error) {
    console.error(error);
  } finally {
    isSaving.value = false;
  }
};
</script>

<template>
  <div class="settings-container">
    <div class="header">
      <h2>Engine Configuration</h2>
    </div>

    <div class="settings-grid">
      
      <div class="settings-card">
        <div class="card-header">
          <h3>Intelligence Engine</h3>
          <p>Select the AI network used for parsing and tailoring.</p>
        </div>
        
        <div class="input-row">
          <div class="input-group">
            <label>Provider</label>
            <select v-model="providerInput" class="custom-select">
              <option v-for="p in providers" :key="p.id" :value="p.id">
                {{ p.name }}
              </option>
            </select>
          </div>

          <div class="input-group">
            <label>Active Model</label>
            <select v-model="modelInput" class="custom-select">
              <option v-for="m in currentModels" :key="m.id" :value="m.id">
                {{ m.name }}
              </option>
            </select>
          </div>
        </div>
      </div>

      <div class="settings-card">
        <div class="card-header">
          <h3>API Credentials</h3>
          <p>Your {{ providerName }} key is encrypted locally using AES-256. It never leaves your machine.</p>
        </div>
        
        <div class="input-group">
          <label>{{ providerName }} Secret Key</label>
          <input 
            v-model="apiKeyInput" 
            type="password" 
            :placeholder="store.hasSecureKey ? '•••••••••••••••• (Key is securely set)' : 'Paste API Key here...'"
            spellcheck="false"
          />
        </div>
      </div>

    </div>

    <div class="actions-footer">
      <transition name="fade">
        <span class="success-msg" v-if="showSuccess">
          <span class="dot"></span> Configuration Saved
        </span>
      </transition>
      
      <button class="btn-primary save-btn" @click="handleSave" :disabled="isSaving">
        {{ isSaving ? 'Securing...' : 'Save Configuration &rarr;' }}
      </button>
    </div>
  </div>
</template>

<style scoped>

/* Keep all your previous styles, just add this one line to place the dropdowns side-by-side */
.input-row { display: flex; gap: 20px; }
.input-row > .input-group { flex: 1; }


.settings-container {
  padding: 40px;
  height: 100%;
  display: flex;
  flex-direction: column;
  max-width: 900px;
  margin: 0 auto;
}

.header { margin-bottom: 40px; }
.header h2 { margin: 0; font-weight: 600; color: #ededed; font-size: 2rem; }

.settings-grid {
  display: flex;
  flex-direction: column;
  gap: 24px;
  flex-grow: 1;
}

.settings-card {
  background: #0a0a0a;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  padding: 30px;
}

.card-header { margin-bottom: 24px; }
.card-header h3 { margin: 0 0 8px 0; color: #ededed; font-size: 1.2rem; }
.card-header p { margin: 0; color: #888; font-size: 0.9rem; line-height: 1.5; }

.input-group {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

label {
  color: #00e599;
  font-weight: 600;
  font-size: 0.85rem;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

input, .custom-select {
  background: #030303;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  padding: 16px;
  color: #ededed;
  font-family: monospace;
  font-size: 1rem;
  outline: none;
  transition: all 0.2s ease;
  width: 100%;
}

input:focus, .custom-select:focus {
  border-color: #00e599;
  box-shadow: 0 0 0 1px rgba(0, 229, 153, 0.3);
}

.custom-select { cursor: pointer; appearance: none; }

/* Actions Footer */
.actions-footer {
  margin-top: 40px;
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 20px;
  padding-top: 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}

.save-btn {
  background-color: #00e599;
  color: #000;
  border: none;
  border-radius: 8px;
  padding: 14px 28px;
  font-weight: 600;
  font-size: 1rem;
  cursor: pointer;
  transition: 0.2s;
}

.save-btn:hover:not(:disabled) { background-color: #00c785; }
.save-btn:disabled { opacity: 0.6; cursor: not-allowed; }

.success-msg {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #00e599;
  font-weight: 500;
  font-size: 0.95rem;
}

.success-msg .dot {
  width: 6px;
  height: 6px;
  background-color: #00e599;
  border-radius: 50%;
  box-shadow: 0 0 8px #00e599;
}

/* Vue transition for the success message */
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>