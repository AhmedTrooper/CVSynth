// Initialize UI and Load Settings
document.addEventListener('DOMContentLoaded', () => {
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

  function loadSettings() {
    chrome.storage.local.get([
      'hostType', 'localPort', 'customHost', 'customPort', 'remoteUrl', 'secret', 'selector'
    ], (result) => {
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
    });
  }

  // Load saved settings initially
  loadSettings();

  // Save Selector
  document.getElementById('saveSelectorBtn').addEventListener('click', () => {
    const selector = selectorInput.value.trim() || 'body';
    chrome.storage.local.set({ selector }, () => {
      showStatus("Selector saved!", "success");
    });
  });

  // Reset Selector
  document.getElementById('resetSelectorBtn').addEventListener('click', () => {
    selectorInput.value = 'body';
    chrome.storage.local.set({ selector: 'body' }, () => {
      showStatus("Selector reset to default.", "neutral");
    });
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
  document.getElementById('saveSettingsBtn').addEventListener('click', () => {
    const hostType = hostTypeSelect.value;
    const localPort = localPortInput.value.trim() || '14207';
    const customHost = customHostInput.value.trim();
    const customPort = customPortInput.value.trim() || '14207';
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
      // Force HTTPS for remote URLs (security best practice)
      if (finalHost) {
        if (finalHost.startsWith('http://')) {
          finalHost = finalHost.replace('http://', 'https://');
        } else if (!finalHost.startsWith('https://')) {
          finalHost = 'https://' + finalHost;
        }
      }
    }

    // Remove trailing slash for consistency
    if (finalHost.endsWith('/')) {
      finalHost = finalHost.slice(0, -1);
    }

    if (!finalHost) {
      showStatus("Please provide a valid Host or URL.", "error");
      return;
    }

    const saveAction = () => {
      chrome.storage.local.set({ 
        host: finalHost, 
        hostType, 
        localPort, 
        customHost, 
        customPort, 
        remoteUrl, 
        secret 
      }, () => {
        showStatus("Settings saved successfully!", "success");
      });
    };

    // If it's a remote URL, we must request permission dynamically
    if (hostType === 'remote' || (hostType === 'customLocal' && customHost !== '127.0.0.1' && customHost !== 'localhost')) {
      const origin = new URL(finalHost).origin + '/*';
      chrome.permissions.request({
        origins: [origin]
      }, (granted) => {
        if (granted) {
          saveAction();
        } else {
          showStatus("Permission denied. Cannot save custom host.", "error");
        }
      });
    } else {
      saveAction();
    }
  });

  // Extract and Send
  document.getElementById('extractBtn').addEventListener('click', () => {
    const selector = selectorInput.value.trim() || 'body';
    
    showStatus("Extracting content...", "neutral");

    chrome.runtime.sendMessage({ action: "START_EXTRACTION", selector }, (response) => {
      if (response && response.success) {
        showStatus("Job ingested into Inbox vault!", "success");
      } else {
        const errorMsg = response?.error || "Connection failed. Is your RoleTect instance reachable?";
        showStatus("Error: " + errorMsg, "error");
      }
    });
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
