import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// Define strict TypeScript interfaces
export interface Job {
  id: string; // Now a String (Slug)
  title: string;
  company: string;
  status: string;
}

export const useJobsStore = defineStore('jobs', () => {
  const jobsList = ref<Job[]>([]);
  const currentJob = ref<Job | null>(null);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  // Actions
  const createNewJob = async (title: string, company: string): Promise<string> => {
    isLoading.value = true;
    error.value = null;
    try {
      // Calls the Rust command, gets the generated nanoid back
      const newSlug: string = await invoke('create_job', { title, company });
      return newSlug; 
    } catch (err: any) {
      error.value = err.toString();
      throw err;
    } finally {
      isLoading.value = false;
    }
  };

  const loadJobDetails = async (slug: string) => {
     // TODO: Invoke rust command to fetch job by slug
  };

  return { 
    jobsList, 
    currentJob, 
    isLoading, 
    error, 
    createNewJob,
    loadJobDetails
  };
});