import { describe, expect, it } from 'vitest'
import type { ProbeFormation } from '@/types'
import {
    AU_KM,
    coordinateForDisplay,
    coordinateFromDisplay,
    moveFormationWithIds,
} from './formationEditorUtils'

function formation(id: number, name: string): ProbeFormation {
    return { id, name, probes: [] }
}

describe('formation editor helpers', () => {
    it('moves formations and swaps ids so the order survives an id-sorted reload', () => {
        const formations = [formation(0, 'Alpha'), formation(1, 'Bravo')]

        expect(moveFormationWithIds(formations, 1, -1)).toBe(true)
        expect(formations.map((item) => [item.id, item.name])).toEqual([
            [0, 'Bravo'],
            [1, 'Alpha'],
        ])

        formations.sort((a, b) => a.id - b.id)
        expect(formations.map((item) => item.name)).toEqual(['Bravo', 'Alpha'])
    })

    it('rejects moves outside the list', () => {
        const formations = [formation(0, 'Alpha')]
        expect(moveFormationWithIds(formations, 0, -1)).toBe(false)
        expect(formations[0].name).toBe('Alpha')
    })

    it('round-trips AU coordinate display values', () => {
        expect(coordinateForDisplay(AU_KM / 4, 'au')).toBe(0.25)
        expect(coordinateFromDisplay(0.25, 'au')).toBe(AU_KM / 4)
        expect(coordinateFromDisplay(12_500, 'km')).toBe(12_500)
    })
})
