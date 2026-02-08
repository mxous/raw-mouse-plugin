// Main entry point for the Tauri app
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

declare global {
  interface Window {
    startListening: () => Promise<void>
    stopListening: () => Promise<void>
    addEvent: (event: any) => void
    invoke: typeof invoke
    isListening: boolean
  }
}

// Start listening function (called from inline onclick)
window.startListening = async function() {
  try {
    await invoke('start_raw_input')
    window.isListening = true
    const statusText = document.getElementById('status-text')
    const statusDiv = document.getElementById('status')
    if (statusText) statusText.textContent = 'Listening'
    if (statusDiv) statusDiv.className = 'status listening'
    console.log('Started listening for raw mouse input')
  } catch (error) {
    console.error('Error starting listener:', error)
    const statusText = document.getElementById('status-text')
    const statusDiv = document.getElementById('status')
    if (statusText) statusText.textContent = `Error: ${error}`
    if (statusDiv) statusDiv.className = 'status stopped'
  }
}

// Stop listening function (called from inline onclick)
window.stopListening = async function() {
  try {
    await invoke('stop_raw_input')
    window.isListening = false
    const statusText = document.getElementById('status-text')
    const statusDiv = document.getElementById('status')
    if (statusText) statusText.textContent = 'Stopped'
    if (statusDiv) statusDiv.className = 'status stopped'
    console.log('Stopped listening')
  } catch (error) {
    console.error('Error stopping listener:', error)
    const statusText = document.getElementById('status-text')
    const statusDiv = document.getElementById('status')
    if (statusText) statusText.textContent = `Error: ${error}`
    if (statusDiv) statusDiv.className = 'status stopped'
  }
}

// Set up event listener
console.log('Setting up device-changed event listener...')
listen('device-changed', (event: any) => {
  const data = event.payload
  if (window.addEvent) {
    window.addEvent(data)
  } else {
    console.error('window.addEvent is not defined')
  }
}).then(() => {
  console.log('Event listener successfully registered')
}).catch(err => {
  console.error('Failed to listen for device-changed:', err)
})
console.log('Main.ts loaded and Tauri API initialized')

// Export invoke for use in HTML onclick handlers
window.invoke = invoke
