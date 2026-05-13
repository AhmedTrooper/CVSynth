import { defineStore } from 'pinia';
import { ref } from 'vue';
import { Stronghold, Store } from '@tauri-apps/plugin-stronghold';
import { appDataDir } from '@tauri-apps/api/path';

export const useSettingsStore = defineStore('settings', () => {
  const hasSecureKey = ref(false);
  const selectedAiModel = ref('gpt-4o');

  // Helper to initialize and unlock the Stronghold vault
  const getVault = async (): Promise<Store> => {
    const dir = await appDataDir();
    const vaultPath = `${dir}/secrets.stronghold`;
    const stronghold = await Stronghold.load(vaultPath, 'cvsynth_background_password');
    return stronghold.getStore('api_client', []);
  };

  const saveApiKey = async (key: string) => {
    try {
      const store = await getVault();
      // Stronghold stores data as byte arrays, so we encode the string
      await store.insert('openai_key', Array.from(new TextEncoder().encode(key)));
      await store.save();
      hasSecureKey.value = true;
    } catch (error) {
      console.error("Failed to save to Stronghold:", error);
    }
  };

  const loadSettings = async () => {
    try {
      const store = await getVault();
      const keyBytes = await store.get('openai_key');
      hasSecureKey.value = keyBytes !== null;
      // We will load the 'selectedAiModel' from SQLite later!
    } catch {
      hasSecureKey.value = false;
    }
  };

  return { hasSecureKey, selectedAiModel, saveApiKey, loadSettings };
});