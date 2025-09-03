// Debug script to run in browser console
// Copy and paste this entire script into the browser console when MindLink is open

console.log('🐛 Starting MindLink Plugin Debug...');

// Clear plugin cache
console.log('🧹 Clearing plugin cache...');
localStorage.removeItem('mindlink_plugin_configs');
Object.keys(localStorage).forEach(key => {
  if (key.startsWith('mindlink_') && (key.includes('token') || key.includes('oauth'))) {
    localStorage.removeItem(key);
    console.log('🗑️ Removed:', key);
  }
});

// Check what's currently loaded
if (window.React && window.React.__SECRET_INTERNALS_DO_NOT_USE_OR_YOU_WILL_BE_FIRED) {
  console.log('⚛️ React is loaded');
}

// Force refresh the ProvidersCard component
console.log('🔄 Attempting to refresh providers...');

// Try to access the plugin system if it's available globally
if (window.__MINDLINK_PLUGINS__) {
  console.log('🔌 Found global plugin registry');
  console.log('📦 Available plugins:', window.__MINDLINK_PLUGINS__);
} else {
  console.log('❌ No global plugin registry found');
}

// Check for any spinning/loading states
const spinningElements = document.querySelectorAll('[class*="spin"], [class*="loading"], .loading-spinner, .provider-status-dot--disconnected');
console.log('🌀 Found potentially spinning elements:', spinningElements.length);
spinningElements.forEach((el, i) => {
  console.log(`  ${i+1}. ${el.className} - ${el.tagName}`);
});

// Look for Ollama specifically
const ollamaElements = document.querySelectorAll('[class*="ollama"], [data-testid*="ollama"]');
if (ollamaElements.length > 0) {
  console.log('🦙 Found Ollama elements:', ollamaElements.length);
  ollamaElements.forEach((el, i) => {
    console.log(`  ${i+1}. ${el.className} - ${el.textContent?.trim()}`);
  });
} else {
  console.log('🦙 No Ollama elements found in DOM');
}

// Try to find provider items
const providerItems = document.querySelectorAll('.provider-item, [class*="provider"]');
console.log('🏪 Found provider items:', providerItems.length);
providerItems.forEach((el, i) => {
  const name = el.querySelector('.provider-name, [class*="name"]')?.textContent?.trim();
  const status = el.querySelector('[class*="status"], [class*="dot"], [class*="icon"]')?.className;
  console.log(`  ${i+1}. ${name || 'Unknown'} - Status: ${status || 'Unknown'}`);
});

// Manual refresh attempt
console.log('🔄 Attempting manual refresh...');
window.location.reload();

console.log('✅ Debug complete - page will refresh');
console.log('💡 After refresh, check if Ollama appears properly connected');