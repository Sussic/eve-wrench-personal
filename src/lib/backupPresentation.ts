export type BackupReasonKey =
    | 'preSelectiveCopy'
    | 'preRestore'
    | 'preFormationEdit'
    | 'preFormationVariants'
    | 'preImport'

const AUTOMATIC_REASONS: Record<string, BackupReasonKey> = {
    'pre-selective-copy': 'preSelectiveCopy',
    'pre-restore': 'preRestore',
    'pre-formation-edit': 'preFormationEdit',
    'pre-formation-variants': 'preFormationVariants',
    'pre-import': 'preImport',
}

export function backupReasonKey(name: string): BackupReasonKey | null {
    const normalized = name.replace(/ \(\d+\)$/, '')
    return AUTOMATIC_REASONS[normalized] ?? null
}

export function formatBackupTimestamp(timestamp: number): string {
    return new Intl.DateTimeFormat(undefined, {
        dateStyle: 'medium',
        timeStyle: 'short',
    }).format(new Date(timestamp * 1000))
}
