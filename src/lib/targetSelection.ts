import type { AppData, ServerId, SettingsEntry, SettingsKind } from '@/types'

export function collectEntriesByKind(
    appData: AppData,
    kind: SettingsKind,
    serverId?: ServerId | null
): SettingsEntry[] {
    const entries: SettingsEntry[] = []
    for (const server of appData.servers) {
        if (serverId && server.info.id !== serverId) continue
        for (const profile of server.profiles) {
            entries.push(
                ...(kind === 'char' ? profile.characters : profile.accounts)
            )
        }
    }
    return entries
}

export function mergeUniqueTargets(
    existing: SettingsEntry[],
    candidates: SettingsEntry[],
    excludedPath: string | null
): SettingsEntry[] {
    const merged = [...existing]
    const paths = new Set(existing.map((entry) => entry.path))
    for (const candidate of candidates) {
        if (candidate.path === excludedPath || paths.has(candidate.path)) {
            continue
        }
        paths.add(candidate.path)
        merged.push(candidate)
    }
    return merged
}
