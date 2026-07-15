import { onMounted, onUnmounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export function useEveRunning(pollIntervalMs = 3000) {
    const eveRunning = ref(false)
    let timer: ReturnType<typeof setInterval> | null = null

    async function refreshEveRunning() {
        try {
            eveRunning.value = await invoke<boolean>('is_eve_running')
        } catch {
            // Process detection is a safety convenience. Backend mutations
            // perform the authoritative check again before writing.
            eveRunning.value = false
        }
    }

    onMounted(() => {
        void refreshEveRunning()
        timer = setInterval(refreshEveRunning, pollIntervalMs)
    })

    onUnmounted(() => {
        if (timer) clearInterval(timer)
    })

    return { eveRunning, refreshEveRunning }
}
