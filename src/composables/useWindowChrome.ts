import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { type UnlistenFn } from '@tauri-apps/api/event'
import { platform } from '@tauri-apps/plugin-os'

// Shared custom-titlebar behavior for every app window. On macOS the native
// traffic lights are overlaid (decorations stay on); on Windows/Linux the
// native frame is removed so the webview draws its own controls.
export function useWindowChrome() {
    const appWindow = getCurrentWindow()
    const isMac = ref(true)
    const isMaximized = ref(false)
    let unlistenResized: UnlistenFn | null = null

    onMounted(async () => {
        isMac.value = platform() === 'macos'
        isMaximized.value = await appWindow.isMaximized()

        if (!isMac.value) {
            await appWindow.setDecorations(false)
        }

        unlistenResized = await appWindow.onResized(async () => {
            isMaximized.value = await appWindow.isMaximized()
        })
    })

    onUnmounted(() => {
        unlistenResized?.()
    })

    return {
        isMac,
        isMaximized,
        minimize: () => appWindow.minimize(),
        toggleMaximize: () => appWindow.toggleMaximize(),
        close: () => appWindow.close(),
    }
}
