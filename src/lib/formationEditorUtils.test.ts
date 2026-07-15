import { describe, expect, it } from 'vitest'
import type { ProbeFormation } from '@/types'
import {
    AU_KM,
    coordinateForDisplay,
    coordinateFromDisplay,
    centredProbePreview,
    deleteFormationAfterConfirmation,
    formationCentroid,
    moveFormationWithIds,
    probeCompatibility,
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

    it('centres an effective EVE preview without mutating the edit values', () => {
        const probes = [
            { x: 100, y: 0, z: 0, range: 0.5 },
            { x: 300, y: 0, z: 0, range: 0.5 },
        ]
        const preview = centredProbePreview(probes)
        expect(formationCentroid(preview)).toEqual({ x: 0, y: 0, z: 0 })
        expect(probes[0].x).toBe(100)
    })

    it('derives probe compatibility from the stored ranges', () => {
        const at = (range: number) => [{ x: 0, y: 0, z: 0, range }]
        expect(probeCompatibility(at(0.25))).toBe('core')
        expect(probeCompatibility(at(64))).toBe('combat')
        expect(probeCompatibility(at(8))).toBe('both')
        expect(probeCompatibility([...at(0.25), ...at(64)])).toBe(
            'incompatible'
        )
    })

    it('does not delete a formation while confirmation is pending or cancelled', async () => {
        const formations = [formation(0, 'Alpha'), formation(1, 'Bravo')]
        let answer: ((confirmed: boolean) => void) | undefined
        const confirmation = new Promise<boolean>((resolve) => {
            answer = resolve
        })

        const deletion = deleteFormationAfterConfirmation(
            formations,
            0,
            () => confirmation
        )

        expect(formations.map((item) => item.name)).toEqual(['Alpha', 'Bravo'])
        answer?.(false)
        expect(await deletion).toBeNull()
        expect(formations.map((item) => item.name)).toEqual(['Alpha', 'Bravo'])
    })

    it('deletes the staged formation only after confirmation succeeds', async () => {
        const formations = [formation(0, 'Alpha'), formation(1, 'Bravo')]

        const deletedIndex = await deleteFormationAfterConfirmation(
            formations,
            0,
            async () => true
        )

        expect(deletedIndex).toBe(0)
        expect(formations.map((item) => item.name)).toEqual(['Bravo'])
    })
})
