// content.js: Runs inside the webpage context to read the DOM safely (Chrome)
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.action === "GET_DOM") {
    try {
      const targetElement = document.querySelector(request.selector);

      if (!targetElement) {
        sendResponse({
          success: false,
          error: `Selector '${request.selector}' not found on this page.`,
        });
        return true;
      }

      // Extract the data cleanly and squash it for AI token efficiency
      const cleanData = extractStructuredData(targetElement);

      sendResponse({
        success: true,
        url: window.location.href,
        html: cleanData,
      });
    } catch (err) {
      sendResponse({ success: false, error: err.message });
    }
  }
  return true; // Required for Chrome to keep the message channel open asynchronously
});

/**
 * Cleans the DOM node and extracts a highly compressed, token-efficient string.
 */
function extractStructuredData(element) {
  // 1. Clone the element so we don't accidentally mutate the live webpage
  const clone = element.cloneNode(true);

  // 2. Remove noisy tags that confuse AI and bloat the payload
  const noiseSelectors =
    "script, style, noscript, svg, img, iframe, nav, footer, button, .visually-hidden";
  clone.querySelectorAll(noiseSelectors).forEach((el) => el.remove());

  // 3. Create an array to hold the extracted text blocks
  let structuredContent = [];

  // 4. Walk through the cleaned DOM and grab text from block elements
  const blockElements = clone.querySelectorAll(
    "h1, h2, h3, h4, p, li, article, section",
  );

  if (blockElements.length > 0) {
    blockElements.forEach((el) => {
      // Get text and skip empty blocks
      let text = el.innerText.trim();
      if (!text) return;

      // Push raw text
      structuredContent.push(text);
    });
  } else {
    // Fallback if the site uses non-semantic formatting (no p, h1, li tags)
    structuredContent.push(clone.innerText.trim());
  }

  // 5. Join all extracted sections with a period and space
  let finalString = structuredContent.join(". ");

  // 6. The Ultimate Token-Squashing RegEx Pipeline
  return finalString
    .replace(/[\n\r]+/g, ". ") // Turns ANY newline or carriage return into a period
    .replace(/\s+/g, " ") // Squashes massive horizontal gaps into 1 single space
    .replace(/\.{2,}/g, ".") // Squashes weird double periods (e.g. "...") into 1 period
    .replace(/\. \./g, ".") // Cleans up messy period-space-period gaps
    .trim(); // Chops off any spaces at the very beginning or end
}
