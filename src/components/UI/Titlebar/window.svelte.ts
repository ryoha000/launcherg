import type { Window } from '@tauri-apps/api/window'
import { window } from '@tauri-apps/api'

export function useWindow() {
  let unlistenResize: (() => void) | null = null
  let unlistenFocus: (() => void) | null = null
  let appWindow = $state<Window | null>(null)
  let isMaximized = $state(false)
  let isFocused = $state(true)

  const initialize = async () => {
    if (typeof window !== 'undefined') {
      appWindow = window.getCurrentWindow()
      unlistenResize = await appWindow.onResized(async () => {
        isMaximized = await appWindow?.isMaximized() ?? false
      })
      unlistenFocus = await appWindow.onFocusChanged(async () => {
        isFocused = await appWindow?.isFocused() ?? false
      })
    }
  }
  const cleanup = () => {
    if (unlistenResize) {
      unlistenResize()
      unlistenResize = null
    }
    if (unlistenFocus) {
      unlistenFocus()
      unlistenFocus = null
    }
  }

  const minimize = () => {
    appWindow?.minimize()
  }
  const toggleMaximize = () => {
    appWindow?.toggleMaximize()
  }
  const close = () => {
    appWindow?.close()
  }

  return {
    isMaximized: () => isMaximized,
    isFocused: () => isFocused,
    initialize,
    cleanup,
    minimize,
    toggleMaximize,
    close,
  }
}
