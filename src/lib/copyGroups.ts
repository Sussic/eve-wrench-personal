import type { SettingsKind } from '@/types'

// Mirrors the group rules in src-tauri/src/evesettings.rs (group_rule) —
// keep the id sets in sync. defaultOn is false for groups people
// legitimately customize per identity and would be annoyed to have
// overwritten (e.g. HUD module slot layout, typed search history).
export interface CopyGroup {
    id: string
    kinds: SettingsKind[]
    defaultOn: boolean
}

const COPY_GROUPS: CopyGroup[] = [
    // Account (core_user) groups
    { id: 'overview', kinds: ['user'], defaultOn: true },
    { id: 'probes', kinds: ['user'], defaultOn: true },
    { id: 'suppress', kinds: ['user'], defaultOn: true },
    { id: 'audio', kinds: ['user'], defaultOn: true },
    { id: 'camera_graphics', kinds: ['user'], defaultOn: true },
    { id: 'market', kinds: ['user'], defaultOn: true },
    { id: 'slots', kinds: ['user'], defaultOn: false },
    { id: 'tabgroups', kinds: ['user'], defaultOn: true },
    // Character (core_char) groups
    { id: 'windows', kinds: ['char'], defaultOn: true },
    { id: 'neocom', kinds: ['char'], defaultOn: true },
    { id: 'chat', kinds: ['char'], defaultOn: true },
    { id: 'infopanels', kinds: ['char'], defaultOn: true },
    { id: 'dockpanels', kinds: ['char'], defaultOn: true },
    // Present in both file kinds
    { id: 'search_history', kinds: ['user', 'char'], defaultOn: false },
]

export function groupsForKind(kind: SettingsKind): CopyGroup[] {
    return COPY_GROUPS.filter((g) => g.kinds.includes(kind))
}

export function defaultGroupSelection(): Record<string, boolean> {
    return Object.fromEntries(COPY_GROUPS.map((g) => [g.id, g.defaultOn]))
}
