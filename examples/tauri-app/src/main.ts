// Main entry point for the Tauri app
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

declare global {
  interface Window {
    startListening: () => Promise<void>
    stopListening: () => Promise<void>
    addEvent: (event: any) => void
    setMode: (mode: string) => Promise<void>
    setCurrentMode: (mode: string) => void
    invoke: typeof invoke
    isListening: boolean
  }
}

let isListening = false
let poller: number | null = null
let currentMode = 'relative'

function startPoller() {
  stopPoller()
  poller = window.setInterval(async () => {
    try {
      const [x, y] = await invoke<[number, number]>('get_mouse_position')
      if (window.addEvent) {
        window.addEvent({ kind: 'MouseMove', value: { x, y } })
      }
    } catch (e) {
      console.error('Polling error:', e)
    }
  }, 16) // ~60fps
}

function stopPoller() {
  if (poller !== null) {
    clearInterval(poller)
    poller = null
  }
}

// Set tracking mode function
window.setMode = async function(mode: string) {
  try {
    await invoke('set_tracking_mode', { mode })
    currentMode = mode

    // Update the inline script's currentMode variable
    if (window.setCurrentMode) {
      window.setCurrentMode(mode)
    }

    // Update button UI
    document.getElementById('mode-relative')?.classList.toggle('active', mode === 'relative')
    document.getElementById('mode-absolute')?.classList.toggle('active', mode === 'absolute')

    // Update info text
    const infoEl = document.getElementById('mode-info')
    if (infoEl) {
      if (mode === 'relative') {
        infoEl.textContent = 'Relative mode - values are deltas from raw input'
      } else {
        infoEl.textContent = 'Absolute mode - values are monitor coordinates'
      }
    }

    // Reset display
    const coordX = document.getElementById('coord-x')
    const coordY = document.getElementById('coord-y')
    if (coordX) coordX.textContent = '0'
    if (coordY) coordY.textContent = '0'

    // Switch between hook events and polling while listening
    if (isListening) {
      if (mode === 'absolute') {
        startPoller()
      } else {
        stopPoller()
      }
    }

    console.log('Tracking mode set to:', mode)
  } catch (error) {
    console.error('Error setting tracking mode:', error)
  }
}

// Start listening function (called from inline onclick)
window.startListening = async function() {
  try {
    await invoke('start_raw_input')
    isListening = true
    window.isListening = true
    const statusText = document.getElementById('status-text')
    const statusDiv = document.getElementById('status')
    if (statusText) statusText.textContent = 'Listening'
    if (statusDiv) statusDiv.className = 'status listening'

    if (currentMode === 'absolute') {
      startPoller()
    }

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
    stopPoller()
    await invoke('stop_raw_input')
    isListening = false
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
  // Call the global addEvent function from HTML
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


