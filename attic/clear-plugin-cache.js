// Clear plugin cache script for MindLink
console.log('🧹 Clearing MindLink plugin cache...')

// Clear localStorage keys related to plugins
const keysToRemove = [
  'mindlink_plugin_configs',
  'mindlink_ollama_token',
  'mindlink_openai_token', 
  'mindlink_anthropic_token',
  'mindlink_google_token'
]

keysToRemove.forEach(key => {
  try {
    localStorage.removeItem(key)
    console.log('✅ Cleared:', key)
  } catch (e) {
    console.log('❌ Failed to clear:', key, e.message)
  }
})

// Clear any OAuth state
Object.keys(localStorage).forEach(key => {
  if (key.includes('oauth_state') && key.includes('mindlink')) {
    localStorage.removeItem(key)
    console.log('✅ Cleared OAuth state:', key)
  }
})

console.log('🎯 Plugin cache cleared! Refresh the page to see changes.')
console.log('📝 Run this script in the browser console: copy and paste the above code')