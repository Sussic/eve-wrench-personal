import { describe, expect, it } from 'vitest'
import type { BackupEntry, ProfileData, SettingsEntry } from '@/types'
import { backupMatches, settingsEntryMatches } from './settingsSearch'

const character: SettingsEntry = {
    path: 'char.dat',
    id: '90000001',
    kind: 'char',
    server: 'tranquility',
    profile: 'Default',
    display_name: 'Sussic Seven',
    character: {
        name: 'Sussic Seven',
        corporation: 'Signal Cartel',
        portrait_url: '',
    },
    alias: null,
    modified_time: 1,
    relative_time: 'now',
}

const profile: ProfileData = {
    name: 'Default',
    path: 'settings_Default',
    accounts: [],
    characters: [character],
}

describe('settings search', () => {
    it('matches multiple tokens across character and corporation fields', () => {
        expect(settingsEntryMatches(character, profile, 'sussic signal')).toBe(
            true
        )
        expect(
            settingsEntryMatches(character, profile, 'sussic pandemic')
        ).toBe(false)
    })

    it('matches backup identity, server, and profile context', () => {
        const backup: BackupEntry = {
            id: 'backup-path',
            name: 'before roam',
            path: 'backup-path',
            timestamp: 1,
            kind: 'char',
            original_id: character.id,
            original_name: character.display_name,
            server: 'tranquility',
            profile: 'Default',
            display_name: 'before roam',
            relative_time: 'now',
        }
        expect(backupMatches(backup, 'roam default')).toBe(true)
        expect(backupMatches(backup, 'singularity')).toBe(false)
    })
})
