import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export interface BaseResume {
  id: string; // Slug
  name: string;
  category: string;
  latexContent: string;
}

export const useResumesStore = defineStore('resumes', () => {
  const baseResumes = ref<BaseResume[]>([]);
  
  const loadBaseResumes = async () => {
    // TODO: Fetch from SQLite
  };

  return { baseResumes, loadBaseResumes };
});