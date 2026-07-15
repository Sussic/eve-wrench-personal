import type { BackupEntry, ProfileData, SettingsEntry } from '@/types'
import { getServerShortName } from '@/types'

function tokens(query: string): string[] {
    return query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean)
}

function matches(haystack: Array<string | null | undefined>, query: string) {
    const wanted = tokens(query)
    if (!wanted.length) return true
    const searchable = haystack.filter(Boolean).join(' ').toLocaleLowerCase()
    return wanted.every((token) => searchable.includes(token))
}

export function settingsEntryMatches(
    entry: SettingsEntry,
    profile: ProfileData,
    query: string
): boolean {
    return matches(
        [
            entry.display_name,
            entry.id,
            entry.alias,
            entry.character?.name,
            entry.character?.corporation,
            profile.name,
            entry.server,
            getServerShortName(entry.server),
        ],
        query
    )
}

export function backupMatches(backup: BackupEntry, query: string): boolean {
    return matches(
        [
            backup.name,
            backup.original_name,
            backup.original_id,
            backup.server,
            getServerShortName(backup.server),
            backup.profile,
            backup.kind === 'char' ? 'character toon' : 'account user',
        ],
        query
    )
}
