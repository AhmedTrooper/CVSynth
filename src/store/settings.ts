import { defineStore } from 'pinia';
import { ref } from 'vue';
import { Stronghold } from '@tauri-apps/plugin-stronghold';
import { appDataDir } from '@tauri-apps/api/path';
import { invoke } from '@tauri-apps/api/core';

export const useSettingsStore = defineStore('settings', () => {
  const hasSecureKey = ref(false);
  const selectedAiProvider = ref('openai'); // Ready for Gemini, Groq, etc.
  const selectedAiModel = ref('gpt-4o');

  // Helper to load the Stronghold instance and the Store
  const getVault = async () => {
    const dir = await appDataDir();
    const vaultPath = `${dir}/secrets.stronghold`;
    const stronghold = await Stronghold.load(vaultPath, 'cvsynth_background_password');
    // We must load a client first, then get the store
    const client = await stronghold.loadClient('api_client');
    const store = client.getStore();
    
    return { stronghold, store };
  };

  const saveApiKey = async (key: string) => {
    try {
      const { stronghold, store } = await getVault();
      await store.insert('ai_api_key', Array.from(new TextEncoder().encode(key)));
      // Save is called on the stronghold instance!
      await stronghold.save(); 
      hasSecureKey.value = true;
    } catch (error) {
      console.error("Stronghold save error:", error);
      throw error;
    }
  };

  const getDecryptedKey = async (): Promise<string | null> => {
    try {
      const { store } = await getVault();
      const keyBytes = await store.get('ai_api_key');
      if (keyBytes) {
        return new TextDecoder().decode(new Uint8Array(keyBytes as Iterable<number>));
      }
      return null;
    } catch {
      return null;
    }
  };




  const saveModelConfig = async (provider: string, model: string) => {
    try {
      // Pass both to Rust
      await invoke('save_model_pref', { provider, model }); 
      selectedAiProvider.value = provider;
      selectedAiModel.value = model;
    } catch (error) {
      console.error("SQLite save error:", error);
      throw error;
    }
  };

  const loadSettings = async () => {
    try {
      // Rust now returns the AiConfig object
      const config: { provider: string, model: string } = await invoke('get_model_pref');
      selectedAiProvider.value = config.provider;
      selectedAiModel.value = config.model;

      const { store } = await getVault();
      const keyBytes = await store.get('ai_api_key');
      hasSecureKey.value = keyBytes !== null;
    } catch (e) {
      console.error("Error loading settings:", e);
      hasSecureKey.value = false;
    }
  };

  return { 
    hasSecureKey, 
    selectedAiProvider,
    selectedAiModel, 
    saveApiKey, 
    getDecryptedKey,
    saveModelConfig,
    loadSettings 
  };
});