<script setup lang="ts">
import { ref, onMounted, watch, computed, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { save, open as openDialog } from '@tauri-apps/plugin-dialog';
import { join } from '@tauri-apps/api/path';
import { 
  writeFile, 
  readDir, 
  readTextFile, 
  mkdir, 
  remove, 
  exists
} from '@tauri-apps/plugin-fs';
import { Motion, AnimatePresence } from 'motion-v';
import { useSettingsStore } from '../store/settings';
import { useDialogStore } from '../store/dialog';
import mermaid from 'mermaid';
import svgPanZoom from 'svg-pan-zoom';

// Markdown imports
import MarkdownIt from 'markdown-it';
import DOMPurify from 'dompurify';
import markdownItKatex from 'markdown-it-katex';
import highlightjs from 'highlight.js';
import 'github-markdown-css/github-markdown-dark.css';
import 'highlight.js/styles/github-dark.css';
import 'katex/dist/katex.min.css';

import { 
  Download, 
  RotateCw, 
  X,
  FileCode,
  FolderOpen,
  File,
  ChevronRight,
  ChevronDown,
  Plus,
  Trash2,
  FolderPlus,
  Share2,
  Workflow,
  Maximize2,
  Layout,
  Eye,
  Type
} from '@lucide/vue';

// Codemirror imports
import { Codemirror } from 'vue-codemirror';
import { oneDark } from '@codemirror/theme-one-dark';
import { EditorView } from '@codemirror/view';

const settingsStore = useSettingsStore();
const dialog = useDialogStore();

// Markdown Init
const md = new MarkdownIt({
  html: true,
  linkify: true,
  typographer: true,
  highlight: (str, lang) => {
    if (lang && highlightjs.getLanguage(lang)) {
      try {
        return highlightjs.highlight(str, { language: lang }).value;
      } catch (__) {}
    }
    return ''; // use external default escaping
  }
}).use(markdownItKatex);

// Mermaid Init
mermaid.initialize({
  startOnLoad: false,
  theme: 'dark',
  securityLevel: 'loose',
  flowchart: { useMaxWidth: false, htmlLabels: true }
});

// Codemirror Extensions
const extensions = [
  oneDark,
  EditorView.lineWrapping
];

// Types
interface FileItem {
  name: string;
  path: string;
  isDir: boolean;
  children?: FileItem[];
  isOpen?: boolean;
}

// State
const workspacePath = ref<string | null>(null);
const fileTree = ref<FileItem[]>([]);
const activeFilePath = ref<string | null>(null);
const diagramCode = ref('graph TD\n    A[Start] --> B{Process}\n    B -->|Success| C[End]\n    B -->|Failure| D[Retry]');

const isSidebarVisible = ref(true);
const sidebarWidth = ref(240);
const isResizing = ref(false);

const diagramSvg = ref('');
const markdownHtml = ref('');
const isRendering = ref(false);
const renderingError = ref<string | null>(null);
const isDirty = ref(false);
const editorContainer = ref<HTMLElement | null>(null);
const previewContainer = ref<HTMLElement | null>(null);
const isLoadingWorkspace = ref(false);
const panZoomInstance = ref<any>(null);

const activeTooltip = ref<string | null>(null);

const isMarkdown = computed(() => activeFilePath.value?.endsWith('.md'));

// Persistence & Initialization
onMounted(async () => {
  try {
    const savedWorkspace = await invoke<string | null>('get_diagram_workspace_path');
    if (savedWorkspace && await exists(savedWorkspace)) {
      workspacePath.value = savedWorkspace;
      await refreshFileTree();

      const lastFile = await invoke<string | null>('get_last_opened_diagram');
      if (lastFile && await exists(lastFile)) {
        await selectFile({ name: lastFile.split(/[/\\]/).pop() || '', path: lastFile, isDir: false });
      }
    }
    
    await renderContent();
    setTimeout(() => { isDirty.value = false; }, 100);
  } catch (err) {
    console.error('Failed to initialize Diagram Studio:', err);
  }
});

// Sidebar methods
const toggleSidebar = () => isSidebarVisible.value = !isSidebarVisible.value;
const startResizing = () => {
  isResizing.value = true;
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', stopResizing);
};
const handleMouseMove = (e: MouseEvent) => {
  if (!isResizing.value) return;
  if (e.clientX > 150 && e.clientX < 500) sidebarWidth.value = e.clientX;
};
const stopResizing = () => {
  isResizing.value = false;
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', stopResizing);
};

// Workspace Management
const selectWorkspace = async () => {
  try {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      title: 'Select Diagram Workspace'
    });

    if (selected && typeof selected === 'string') {
      workspacePath.value = selected;
      await invoke('save_diagram_workspace_path', { path: selected });
      await refreshFileTree();
    }
  } catch (err) {
    console.error('Failed to select workspace:', err);
  }
};

const refreshFileTree = async () => {
  if (!workspacePath.value) return;
  isLoadingWorkspace.value = true;
  try {
    fileTree.value = await scanDirectory(workspacePath.value);
  } catch (err) {
    console.error('Failed to scan workspace:', err);
  } finally {
    isLoadingWorkspace.value = false;
  }
};

const scanDirectory = async (dir: string): Promise<FileItem[]> => {
  const entries = await readDir(dir);
  const items: FileItem[] = [];

  for (const entry of entries) {
    const fullPath = await join(dir, entry.name);
    const isDir = entry.isDirectory;
    
    // Support .mmd and .md
    if (!isDir && !entry.name.endsWith('.mmd') && !entry.name.endsWith('.md')) continue;

    items.push({
      name: entry.name,
      path: fullPath,
      isDir: isDir,
      isOpen: false,
      children: isDir ? [] : undefined
    });
  }

  return items.sort((a, b) => {
    if (a.isDir && !b.isDir) return -1;
    if (!a.isDir && b.isDir) return 1;
    return a.name.localeCompare(b.name);
  });
};

const toggleFolder = async (item: FileItem) => {
  item.isOpen = !item.isOpen;
  if (item.isOpen && item.children?.length === 0) {
    item.children = await scanDirectory(item.path);
  }
};

const selectFile = async (item: FileItem) => {
  if (item.isDir) return;
  
  if (isDirty.value && activeFilePath.value) {
    await saveActiveFile();
  }

  try {
    const content = await readTextFile(item.path);
    diagramCode.value = content;
    activeFilePath.value = item.path;
    isDirty.value = false;
    await invoke('save_last_opened_diagram', { path: item.path });
    await renderContent();
  } catch (err) {
    console.error('Failed to read file:', err);
  }
};

const saveActiveFile = async () => {
  if (!activeFilePath.value) return;

  try {
    await writeFile(activeFilePath.value, new TextEncoder().encode(diagramCode.value));
    isDirty.value = false;
  } catch (err) {
    console.error('Failed to save file:', err);
  }
};

const createNewFile = async (parent: FileItem | null = null, ext = '.mmd') => {
  const dir = parent ? parent.path : workspacePath.value;
  if (!dir) return;

  const fileName = await dialog.showPrompt(`Enter name (e.g. flow${ext}):`, '', 'New File');
  if (!fileName) return;

  const fullPath = await join(dir, fileName);
  const initialContent = ext === '.md' ? '# New Document\n\n```mermaid\ngraph TD\n  A --> B\n```' : 'graph TD\n    A --> B';
  
  try {
    await writeFile(fullPath, new TextEncoder().encode(initialContent));
    if (parent) {
      parent.isOpen = true;
      parent.children = await scanDirectory(parent.path);
    } else {
      await refreshFileTree();
    }
  } catch (err: any) {
    await dialog.showAlert(err.toString(), 'Failed to create file');
  }
};

const createNewFolder = async (parent: FileItem | null = null) => {
  const dir = parent ? parent.path : workspacePath.value;
  if (!dir) return;

  const folderName = await dialog.showPrompt('Enter folder name:', '', 'New Folder');
  if (!folderName) return;

  const fullPath = await join(dir, folderName);
  try {
    await mkdir(fullPath);
    if (parent) {
      parent.isOpen = true;
      parent.children = await scanDirectory(parent.path);
    } else {
      await refreshFileTree();
    }
  } catch (err: any) {
    await dialog.showAlert(err.toString(), 'Failed to create folder');
  }
};

const deleteItem = async (item: FileItem) => {
  const confirmed = await dialog.showConfirm(`Are you sure you want to delete "${item.name}"?`, 'Delete Item');
  if (!confirmed) return;

  try {
    await remove(item.path, { recursive: item.isDir });
    if (activeFilePath.value === item.path) {
      activeFilePath.value = null;
      diagramCode.value = 'graph TD\n    A --> B';
    }
    await refreshFileTree();
  } catch (err: any) {
    await dialog.showAlert(err.toString(), 'Failed to delete item');
  }
};

const closeWorkspace = async () => {
  const confirmed = await dialog.showConfirm('Close workspace?', 'Close Workspace');
  if (!confirmed) return;

  workspacePath.value = null;
  fileTree.value = [];
  activeFilePath.value = null;
  await invoke('save_diagram_workspace_path', { path: '' });
};

// Rendering Logic
const renderContent = async () => {
  if (!diagramCode.value.trim()) return;
  
  isRendering.value = true;
  renderingError.value = null;
  
  try {
    if (isMarkdown.value) {
      const rawHtml = md.render(diagramCode.value);
      markdownHtml.value = DOMPurify.sanitize(rawHtml);
      diagramSvg.value = '';
      
      await nextTick();
      // Render mermaid inside markdown
      const mermaidNodes = previewContainer.value?.querySelectorAll('.language-mermaid');
      if (mermaidNodes) {
        for (const node of mermaidNodes) {
          const code = node.textContent || '';
          const id = `mermaid-${Math.random().toString(36).substr(2, 9)}`;
          const { svg } = await mermaid.render(id, code);
          const wrapper = document.createElement('div');
          wrapper.className = 'mermaid-rendered-wrapper';
          wrapper.innerHTML = svg;
          node.parentElement?.replaceWith(wrapper);
        }
      }
    } else {
      const id = `mermaid-${Math.random().toString(36).substr(2, 9)}`;
      const { svg } = await mermaid.render(id, diagramCode.value);
      diagramSvg.value = svg;
      markdownHtml.value = '';
      
      await nextTick();
      initializePanZoom();
    }
  } catch (err: any) {
    console.error("Render Error:", err);
    renderingError.value = err.toString();
  } finally {
    isRendering.value = false;
  }
};

const initializePanZoom = () => {
  if (panZoomInstance.value) {
    panZoomInstance.value.destroy();
    panZoomInstance.value = null;
  }

  if (isMarkdown.value) return;

  const svgElement = previewContainer.value?.querySelector('svg');
  if (svgElement) {
    svgElement.style.width = '100%';
    svgElement.style.height = '100%';
    
    panZoomInstance.value = svgPanZoom(svgElement, {
      zoomEnabled: true,
      controlIconsEnabled: true,
      fit: true,
      center: true,
      minZoom: 0.1,
      maxZoom: 10
    });
  }
};

// Auto-save logic
watch(diagramCode, () => {
  isDirty.value = true;
});

const handleBlur = () => {
  if (isDirty.value) {
    saveActiveFile();
    renderContent();
  }
};

const activeFileName = computed(() => {
  if (!activeFilePath.value) return 'unsaved.mmd';
  return activeFilePath.value.split(/[/\\]/).pop() || 'diagram.mmd';
});
</script>

<template>
  <div class="studio-container">
    <header class="studio-header">
      <div class="header-left">
        <button class="toggle-sidebar-btn" @click="toggleSidebar" title="Toggle Sidebar">
          <Layout :size="18" />
        </button>
        <Share2 :size="20" class="header-icon" />
        <h1>Diagram Studio</h1>
        <span v-if="workspacePath" class="workspace-label">
          {{ workspacePath.split(/[/\\]/).pop() }}
        </span>
      </div>
      
      <div class="header-actions">
        <button 
          class="action-btn render-btn" 
          @click="renderContent" 
          :disabled="isRendering || !diagramCode"
        >
          <Workflow v-if="!isRendering" :size="16" />
          <RotateCw v-else :size="16" class="spinner" />
          <span>{{ isMarkdown ? 'Preview' : 'Render' }}</span>
        </button>
      </div>
    </header>

    <main class="studio-main">
      <div class="split-pane">
        <!-- Sidebar File Explorer -->
        <aside v-if="isSidebarVisible" class="workspace-sidebar" :style="{ width: sidebarWidth + 'px' }">
          <div class="sidebar-header">
            <span>EXPLORER</span>
            <div class="header-tools">
              <button @click="refreshFileTree" title="Refresh"><RotateCw :size="12" /></button>
              <button @click="createNewFile(null, '.mmd')" title="New Diagram"><Plus :size="14" /></button>
              <button @click="createNewFile(null, '.md')" title="New Markdown"><FileCode :size="14" /></button>
              <button @click="createNewFolder()" title="New Folder"><FolderPlus :size="14" /></button>
              <button v-if="workspacePath" @click="closeWorkspace" title="Close Workspace" class="close-workspace-btn"><X :size="14" /></button>
            </div>
          </div>

          <div v-if="!workspacePath" class="sidebar-empty">
            <FolderOpen :size="32" />
            <p>No workspace selected</p>
            <button class="btn-primary-sm" @click="selectWorkspace">Open Folder</button>
          </div>

          <div v-else class="file-tree">
            <div v-if="isLoadingWorkspace" class="tree-loading">
              <RotateCw :size="16" class="spinner" />
            </div>
            
            <template v-else>
              <div v-for="item in fileTree" :key="item.path" class="tree-item-wrapper">
                <div 
                  class="tree-item" 
                  :class="{ active: activeFilePath === item.path }"
                  @click="item.isDir ? toggleFolder(item) : selectFile(item)"
                >
                  <div class="item-icon">
                    <template v-if="item.isDir">
                      <ChevronRight v-if="!item.isOpen" :size="14" />
                      <ChevronDown v-else :size="14" />
                    </template>
                    <File v-else :size="14" />
                  </div>
                  <span class="item-name">{{ item.name }}</span>
                  <div class="item-actions">
                    <template v-if="item.isDir">
                      <button @click.stop="createNewFile(item, '.mmd')" title="New Diagram"><Plus :size="12" /></button>
                      <button @click.stop="createNewFile(item, '.md')" title="New Markdown"><FileCode :size="12" /></button>
                    </template>
                    <button class="item-delete" @click.stop="deleteItem(item)" title="Delete"><Trash2 :size="12" /></button>
                  </div>
                </div>
                
                <AnimatePresence>
                  <Motion
                    v-if="item.isDir && item.isOpen"
                    :initial="{ height: 0, opacity: 0 }"
                    :animate="{ height: 'auto', opacity: 1 }"
                    :exit="{ height: 0, opacity: 0 }"
                    class="tree-children"
                  >
                    <div v-for="child in item.children" :key="child.path" class="tree-item-wrapper">
                      <div 
                        class="tree-item sub-item" 
                        :class="{ active: activeFilePath === child.path }"
                        @click="child.isDir ? toggleFolder(child) : selectFile(child)"
                      >
                        <div class="item-icon">
                          <template v-if="child.isDir">
                            <ChevronRight v-if="!child.isOpen" :size="14" />
                            <ChevronDown v-else :size="14" />
                          </template>
                          <File v-else :size="14" />
                        </div>
                        <span class="item-name">{{ child.name }}</span>
                        <div class="item-actions">
                          <template v-if="child.isDir">
                            <button @click.stop="createNewFile(child, '.mmd')" title="New Diagram"><Plus :size="12" /></button>
                            <button @click.stop="createNewFile(child, '.md')" title="New Markdown"><FileCode :size="12" /></button>
                          </template>
                          <button class="item-delete" @click.stop="deleteItem(child)" title="Delete"><Trash2 :size="12" /></button>
                        </div>
                      </div>
                    </div>
                  </Motion>
                </AnimatePresence>
              </div>
            </template>
          </div>
        </aside>

        <!-- Sidebar Resizer -->
        <div v-if="isSidebarVisible" class="sidebar-resizer" @mousedown="startResizing"></div>

        <!-- Editor Section -->
        <section class="editor-section">
          <div class="pane-header">
            <FileCode :size="14" />
            <span>{{ activeFileName }}</span>
            <span v-if="isDirty" class="dirty-indicator">●</span>
          </div>
          <div class="editor-relative-wrapper" ref="editorContainer">
            <codemirror
              v-model="diagramCode"
              placeholder="Enter Mermaid or Markdown code..."
              :style="{ height: '100%' }"
              :autofocus="true"
              :indent-with-tab="true"
              :tab-size="2"
              :extensions="extensions"
              @blur="handleBlur"
              class="mermaid-editor-cm"
            />
          </div>
        </section>

        <!-- Preview Section -->
        <section class="preview-section">
          <div class="pane-header">
            <Layout :size="14" />
            <span>{{ isMarkdown ? 'DOCUMENT PREVIEW' : 'DIAGRAM PREVIEW' }}</span>
          </div>
          <div class="preview-wrapper" ref="previewContainer" :class="{ 'markdown-view': isMarkdown }">
             <div v-if="isMarkdown" class="markdown-body" v-html="markdownHtml"></div>
             <div v-else-if="diagramSvg" v-html="diagramSvg" class="svg-container"></div>
             <div v-else class="empty-preview">
                <Workflow :size="48" />
                <h3>No content rendered</h3>
                <p>Enter Mermaid or Markdown code to see the preview.</p>
             </div>
             
             <div v-if="isRendering" class="render-overlay">
                <RotateCw :size="24" class="spinner" />
             </div>
          </div>
        </section>
      </div>

      <!-- Error Console -->
      <AnimatePresence>
        <Motion
          v-if="renderingError"
          :initial="{ y: 100, opacity: 0 }"
          :animate="{ y: 0, opacity: 1 }"
          :exit="{ y: 100, opacity: 0 }"
          class="error-console"
        >
          <div class="console-header">
            <div class="title">
              <X :size="14" class="error-icon" />
              <span>RENDERING ERROR</span>
            </div>
            <button class="close-btn" @click="renderingError = null">
              <X :size="14" />
            </button>
          </div>
          <pre class="error-logs">{{ renderingError }}</pre>
        </Motion>
      </AnimatePresence>
    </main>
  </div>
</template>

<style scoped>
.studio-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg);
}

.studio-header {
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  background: var(--bg-accent);
  border-bottom: 1px solid var(--line);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.toggle-sidebar-btn {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
  border-radius: 4px;
  transition: 0.15s;
}

.toggle-sidebar-btn:hover {
  background: var(--surface-soft);
  color: var(--ink);
}

.header-icon {
  color: var(--accent);
}

.header-left h1 {
  font-size: 0.9rem;
  font-weight: 700;
  color: var(--ink);
  margin: 0;
  letter-spacing: 0.02em;
}

.workspace-label {
  font-size: 0.7rem;
  background: var(--surface-soft);
  color: var(--muted);
  padding: 2px 8px;
  border-radius: 4px;
  font-weight: 600;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 0.75rem;
  font-weight: 700;
  cursor: pointer;
  transition: 0.2s;
  border: 1px solid var(--line);
  background: var(--surface-soft);
  color: var(--ink);
}

.action-btn:hover:not(:disabled) {
  border-color: var(--muted);
  background: var(--surface);
}

.render-btn {
  background: var(--accent);
  border-color: var(--accent);
  color: white;
}

.studio-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  position: relative;
}

.split-pane {
  flex: 1;
  display: flex;
  min-height: 0;
  position: relative;
}

.workspace-sidebar {
  background: var(--bg-accent);
  border-right: 1px solid var(--line);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  min-width: 150px;
  max-width: 500px;
}

.sidebar-resizer {
  width: 4px;
  cursor: col-resize;
  background: transparent;
  transition: background 0.2s;
  z-index: 10;
  margin-left: -2px;
}

.sidebar-resizer:hover, .sidebar-resizer:active {
  background: var(--accent);
}

.sidebar-header {
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  background: var(--surface);
  border-bottom: 1px solid var(--line);
  font-size: 0.65rem;
  font-weight: 800;
  color: var(--muted);
  letter-spacing: 0.05em;
}

.header-tools {
  display: flex;
  gap: 8px;
}

.header-tools button {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  padding: 2px;
  display: flex;
  align-items: center;
}

.header-tools button:hover {
  color: var(--ink);
}

.close-workspace-btn:hover {
  color: var(--warning) !important;
}

.sidebar-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--muted);
  gap: 12px;
  padding: 20px;
  text-align: center;
}

.sidebar-empty p {
  font-size: 0.75rem;
  margin: 0;
}

.btn-primary-sm {
  background: var(--accent);
  color: white;
  border: none;
  padding: 6px 12px;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 600;
  cursor: pointer;
}

.file-tree {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.tree-item-wrapper {
  display: flex;
  flex-direction: column;
}

.tree-item {
  display: flex;
  align-items: center;
  padding: 4px 12px;
  gap: 8px;
  cursor: pointer;
  transition: 0.1s;
  position: relative;
  user-select: none;
}

.tree-item:hover {
  background: var(--surface-soft);
}

.tree-item.active {
  background: var(--accent-soft);
  color: var(--accent);
}

.item-icon {
  display: flex;
  align-items: center;
  color: var(--muted);
}

.item-name {
  font-size: 0.8rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.item-actions {
  display: flex;
  gap: 4px;
  opacity: 0;
}

.tree-item:hover .item-actions {
  opacity: 1;
}

.item-actions button {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  padding: 2px;
  display: flex;
  align-items: center;
}

.item-actions button:hover {
  color: var(--ink);
}

.tree-children {
  padding-left: 12px;
}

.tree-loading {
  display: flex;
  justify-content: center;
  padding: 20px;
  color: var(--accent);
}

.editor-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  border-right: 1px solid var(--line);
}

.pane-header {
  height: 32px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
  background: var(--bg-accent);
  border-bottom: 1px solid var(--line);
  font-size: 0.65rem;
  font-weight: 800;
  color: var(--muted);
  letter-spacing: 0.05em;
}

.dirty-indicator {
  color: var(--accent);
  font-size: 10px;
  margin-left: -4px;
}

.editor-relative-wrapper {
  flex: 1;
  position: relative;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: #282c34;
}

.mermaid-editor-cm {
  flex: 1;
  width: 100%;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.9rem;
}

:deep(.cm-editor) {
  height: 100%;
  outline: none !important;
}

.preview-section {
  flex: 1.5;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--bg);
}

.preview-wrapper {
  flex: 1;
  position: relative;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
}

.preview-wrapper.markdown-view {
  display: block;
  overflow-y: auto;
  padding: 24px;
  background: #0d1117;
}

.svg-container {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

:deep(svg) {
  max-width: 100%;
  max-height: 100%;
}

:deep(.mermaid-rendered-wrapper) {
  margin: 20px 0;
  background: #161b22;
  padding: 16px;
  border-radius: 8px;
  display: flex;
  justify-content: center;
}

.empty-preview {
  display: flex;
  flex-direction: column;
  align-items: center;
  color: var(--muted);
  gap: 16px;
}

.render-overlay {
  position: absolute;
  top: 12px;
  right: 12px;
  background: var(--surface-soft);
  padding: 8px;
  border-radius: 50%;
  box-shadow: 0 4px 12px rgba(0,0,0,0.2);
}

.error-console {
  position: absolute;
  bottom: 0;
  left: 0;
  width: 100%;
  max-height: 30%;
  background: #1e1e1e;
  border-top: 1px solid var(--warning);
  display: flex;
  flex-direction: column;
  z-index: 50;
}

.console-header {
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  background: rgba(248, 81, 73, 0.1);
  border-bottom: 1px solid rgba(248, 81, 73, 0.2);
}

.error-logs {
  flex: 1;
  margin: 0;
  padding: 12px;
  overflow-y: auto;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.75rem;
  color: #f85149;
  line-height: 1.5;
  white-space: pre-wrap;
}

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@media (max-width: 1024px) {
  .workspace-sidebar {
    width: 200px;
  }
}

@media (max-width: 768px) {
  .split-pane {
    flex-direction: column;
  }
  
  .workspace-sidebar {
    width: 100%;
    height: 180px;
    border-right: none;
    border-bottom: 1px solid var(--line);
  }
  
  .editor-section {
    border-right: none;
    border-bottom: 1px solid var(--line);
    flex: 1;
  }
  
  .preview-section {
    flex: 1.2;
  }
}
</style>
