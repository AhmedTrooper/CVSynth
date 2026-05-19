// Initialize UI and Load Settings (Firefox)
document.addEventListener('DOMContentLoaded', async () => {
  const hostTypeSelect = document.getElementById('hostType');
  const localPortInput = document.getElementById('localPort');
  const customHostInput = document.getElementById('customHost');
  const customPortInput = document.getElementById('customPort');
  const remoteUrlInput = document.getElementById('remoteUrl');
  
  const secretInput = document.getElementById('secret');
  const selectorInput = document.getElementById('selector');
  const statusDiv = document.getElementById('status');

  const hostGroups = {
    localhost: document.getElementById('localhostGroup'),
    customLocal: document.getElementById('customLocalGroup'),
    remote: document.getElementById('remoteGroup')
  };

  // Tab Switching
  document.querySelectorAll('.tab').forEach(tab => {
    tab.addEventListener('click', () => {
      document.querySelectorAll('.tab, .tab-content').forEach(el => el.classList.remove('active'));
      tab.classList.add('active');
      document.getElementById(tab.dataset.tab).classList.add('active');

      // Reload settings on tab change to discard unsaved changes
      loadSettings();
    });
  });

  async function loadSettings() {
    const result = await browser.storage.local.get([
      'hostType', 'localPort', 'customHost', 'customPort', 'remoteUrl', 'secret', 'selector'
    ]);

    if (result.hostType) {
      hostTypeSelect.value = result.hostType;
      updateHostGroups(result.hostType);
    }
    if (result.localPort) localPortInput.value = result.localPort;
    if (result.customHost) customHostInput.value = result.customHost;
    if (result.customPort) customPortInput.value = result.customPort;
    if (result.remoteUrl) remoteUrlInput.value = result.remoteUrl;
    if (result.secret) secretInput.value = result.secret;
    if (result.selector) selectorInput.value = result.selector;
  }

  // Load saved settings initially
  loadSettings();

  // Save Selector
  document.getElementById('saveSelectorBtn').addEventListener('click', async () => {
    const selector = selectorInput.value.trim() || 'body';
    await browser.storage.local.set({ selector });
    showStatus("Selector saved!", "success");
  });

  // Reset Selector
  document.getElementById('resetSelectorBtn').addEventListener('click', async () => {
    selectorInput.value = 'body';
    await browser.storage.local.set({ selector: 'body' });
    showStatus("Selector reset to default.", "neutral");
  });

  // Host Type Change Logic
  hostTypeSelect.addEventListener('change', (e) => {
    updateHostGroups(e.target.value);
  });

  function updateHostGroups(selectedType) {
    Object.keys(hostGroups).forEach(type => {
      hostGroups[type].style.display = (type === selectedType) ? 'block' : 'none';
    });
  }

  // Save Settings
  document.getElementById('saveSettingsBtn').addEventListener('click', async () => {
    const hostType = hostTypeSelect.value;
    const localPort = localPortInput.value.trim() || '14201';
    const customHost = customHostInput.value.trim();
    const customPort = customPortInput.value.trim() || '14201';
    const remoteUrl = remoteUrlInput.value.trim();
    const secret = secretInput.value.trim();

    let finalHost = '';

    if (hostType === 'localhost') {
      finalHost = `http://127.0.0.1:${localPort}`;
    } else if (hostType === 'customLocal') {
      const h = customHost || '127.0.0.1';
      finalHost = `http://${h}:${customPort}`;
    } else if (hostType === 'remote') {
      finalHost = remoteUrl;
      // Ensure it starts with http/https
      if (finalHost && !finalHost.startsWith('http')) {
        finalHost = 'https://' + finalHost;
      }
    }

    // Remove trailing slash
    if (finalHost.endsWith('/')) {
      finalHost = finalHost.slice(0, -1);
    }

    await browser.storage.local.set({ 
      host: finalHost, // Keep 'host' for background.js compatibility
      hostType, 
      localPort, 
      customHost, 
      customPort, 
      remoteUrl, 
      secret 
    });
    showStatus("Settings saved successfully!", "success");
  });

  // Extract and Send
  document.getElementById('extractBtn').addEventListener('click', async () => {
    const selector = selectorInput.value.trim() || 'body';
    
    showStatus("Processing extraction...", "neutral");

    try {
      const response = await browser.runtime.sendMessage({ action: "START_EXTRACTION", selector });
      if (response && response.success) {
        showStatus("Job ingested into Inbox vault!", "success");
      } else {
        throw new Error(response?.error || "Connection failed. Is your RoleTect instance reachable?");
      }
    } catch (err) {
      showStatus("Error: " + err.message, "error");
    }
  });

  function showStatus(msg, type) {
    statusDiv.textContent = msg;
    statusDiv.className = "";
    if (type === "success") statusDiv.classList.add('status-success');
    if (type === "error") statusDiv.classList.add('status-error');
    if (type === "neutral") {
      statusDiv.style.display = "block";
      statusDiv.style.color = "var(--text)";
    }
  }
});
