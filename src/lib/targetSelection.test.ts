import { describe, expect, it } from 'vitest'
import type {
    AppData,
    ProfileData,
    ServerData,
    ServerId,
    SettingsEntry,
    SettingsKind,
} from '@/types'
import { collectEntriesByKind, mergeUniqueTargets } from './targetSelection'

function entry(
    path: string,
    kind: SettingsKind,
    server: ServerId = 'tranquility'
): SettingsEntry {
    return {
        path,
        id: path,
        kind,
        server,
        profile: 'default',
        display_name: path,
        character: null,
        alias: null,
        modified_time: 0,
        relative_time: '',
    }
}

function profile(
    accounts: SettingsEntry[],
    characters: SettingsEntry[]
): ProfileData {
    return {
        name: 'default',
        path: 'profile',
        accounts,
        characters,
    }
}

function server(id: ServerId, profiles: ProfileData[]): ServerData {
    return {
        info: {
            id,
            name: id,
            short_name: id,
            color: '',
            supports_esi: true,
            brackets_always_show: false,
            server_path: 'server',
        },
        profiles,
    }
}

describe('bulk target selection', () => {
    const accountA = entry('account-a', 'user')
    const accountB = entry('account-b', 'user', 'singularity')
    const characterA = entry('character-a', 'char')
    const characterB = entry('character-b', 'char', 'singularity')
    const appData: AppData = {
        servers: [
            server('tranquility', [profile([accountA], [characterA])]),
            server('singularity', [profile([accountB], [characterB])]),
        ],
        backups: [],
    }

    it('collects every matching entry across servers and profiles', () => {
        expect(collectEntriesByKind(appData, 'user')).toEqual([
            accountA,
            accountB,
        ])
        expect(collectEntriesByKind(appData, 'char')).toEqual([
            characterA,
            characterB,
        ])
    })

    it('limits bulk collection to the requested server', () => {
        expect(collectEntriesByKind(appData, 'char', 'tranquility')).toEqual([
            characterA,
        ])
    })

    it('excludes the source and does not add duplicate targets', () => {
        expect(
            mergeUniqueTargets([accountB], [accountA, accountB], accountA.path)
        ).toEqual([accountB])
    })
})
