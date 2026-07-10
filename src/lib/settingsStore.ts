import { load } from '@tauri-apps/plugin-store'

// Single source for the app's persistent settings (shared across windows via
// the Tauri store plugin, which caches the handle per webview).
const SETTINGS_FILE = 'settings.json'

export function settingsStore() {
    return load(SETTINGS_FILE)
}

// Whether to back up files before mutating them. Defaults on, and stays on if
// the store can't be read (safer to over-back-up than to skip silently).
export async function getAutoBackup(): Promise<boolean> {
    try {
        const store = await settingsStore()
        return (await store.get<boolean>('autoBackup')) ?? true
    } catch {
        return true
    }
}

export async function setAutoBackup(enabled: boolean): Promise<void> {
    try {
        const store = await settingsStore()
        await store.set('autoBackup', enabled)
    } catch {
        // keep the in-memory value even if persistence fails
    }
}
