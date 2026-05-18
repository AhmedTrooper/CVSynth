// Runs inside the webpage context to read the DOM safely
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.action === "GET_DOM") {
    try {
      const targetElement = document.querySelector(request.selector);

      if (!targetElement) {
        sendResponse({ success: false, error: `Selector '${request.selector}' not found on this page.` });
        return true;
      }

      // We send back the text content if it's a large section, 
      // or outerHTML if the user specifically targeted a div.
      // The backend AI is good at handling both.
      sendResponse({
        success: true,
        url: window.location.href,
        html: targetElement.innerText || targetElement.textContent || targetElement.outerHTML
      });
    } catch (err) {
      sendResponse({ success: false, error: err.message });
    }
  }
  return true;
});
