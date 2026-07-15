import { describe, expect, it } from 'vitest'
import { AU_KM } from './formationEditorUtils'
import {
    ALL_SCAN_RANGES,
    buildDirectionalLadder,
    buildDrifterIFlat,
    buildDrifterOFlat,
    buildGridRing,
    buildGridShell,
    buildPinpoint,
    buildSpread,
    buildTetrahedralScan,
    scanRangesFor,
} from './formationPresets'

function distance(probe: { x: number; y: number; z: number }): number {
    return Math.hypot(probe.x, probe.y, probe.z)
}

function centroid(probes: Array<{ x: number; y: number; z: number }>) {
    return probes.reduce(
        (sum, probe) => ({
            x: sum.x + probe.x / probes.length,
            y: sum.y + probe.y / probes.length,
            z: sum.z + probe.z / probes.length,
        }),
        { x: 0, y: 0, z: 0 }
    )
}

describe('formation preset geometry', () => {
    it('keeps Core and Combat probe ranges distinct', () => {
        expect(scanRangesFor('core')).toContain(0.25)
        expect(scanRangesFor('core')).not.toContain(64)
        expect(scanRangesFor('combat')).not.toContain(0.25)
        expect(scanRangesFor('combat')).toContain(64)
        expect(ALL_SCAN_RANGES).toEqual([0.25, 0.5, 1, 2, 4, 8, 16, 32, 64])
    })

    it('builds an eight-probe pinpoint with a central probe and equal outer radius', () => {
        const probes = buildPinpoint(0.5)
        const radius = (0.5 * AU_KM) / 2

        expect(probes).toHaveLength(8)
        expect(distance(probes[0])).toBe(0)
        for (const probe of probes.slice(1)) {
            expect(distance(probe)).toBeCloseTo(radius, 5)
            expect(probe.range).toBe(0.5)
        }
    })

    it('builds a symmetric eight-corner spread', () => {
        const probes = buildSpread(8)
        const component = (8 * AU_KM) / 2

        expect(probes).toHaveLength(8)
        expect(new Set(probes.map((probe) => probe.x))).toEqual(
            new Set([-component, component])
        )
        expect(new Set(probes.map((probe) => probe.y))).toEqual(
            new Set([-component, component])
        )
        expect(new Set(probes.map((probe) => probe.z))).toEqual(
            new Set([-component, component])
        )
    })

    it('builds a regular inner tetrahedron with a centred inverse outer set', () => {
        const probes = buildTetrahedralScan(0.5)
        const inner = probes.slice(0, 4)
        const outer = probes.slice(4)

        expect(probes).toHaveLength(8)
        for (const probe of inner) {
            expect(distance(probe)).toBeCloseTo(0.5 * AU_KM * 0.4, 5)
        }
        for (const probe of outer) {
            expect(distance(probe)).toBeCloseTo(0.5 * AU_KM * 0.5, 5)
        }
        for (let i = 0; i < inner.length; i++) {
            for (let j = i + 1; j < inner.length; j++) {
                const dot =
                    inner[i].x * inner[j].x +
                    inner[i].y * inner[j].y +
                    inner[i].z * inner[j].z
                const cosine = dot / (distance(inner[i]) * distance(inner[j]))
                expect(cosine).toBeCloseTo(-1 / 3, 10)
            }
        }
        expect(centroid(inner)).toEqual({ x: 0, y: 0, z: 0 })
        expect(centroid(outer)).toEqual({ x: 0, y: 0, z: 0 })
    })

    it('keeps every ring and shell bookmark at the requested distance', () => {
        for (const probes of [
            buildGridRing(250, 0.5),
            buildGridShell(250, 0.5),
        ]) {
            expect(probes).toHaveLength(8)
            for (const probe of probes) {
                expect(distance(probe)).toBeCloseTo(250, 8)
            }
        }
    })

    it('builds centred mirrored ladders on the selected axis', () => {
        const probes = buildDirectionalLadder('upDown', 250, 0.5)

        expect(probes.map((probe) => probe.y)).toEqual([
            -1000, -750, -500, -250, 250, 500, 750, 1000,
        ])
        expect(probes.every((probe) => probe.x === 0 && probe.z === 0)).toBe(
            true
        )
        expect(centroid(probes)).toEqual({ x: 0, y: 0, z: 0 })
    })

    it('keeps every on-grid layout centred on the formation origin', () => {
        for (const probes of [
            buildGridRing(250, 0.5),
            buildGridShell(250, 0.5),
            buildDirectionalLadder('northSouth', 250, 0.5),
            buildDirectionalLadder('westEast', 250, 0.5),
            buildDirectionalLadder('upDown', 250, 0.5),
        ]) {
            const centre = centroid(probes)
            expect(centre.x).toBeCloseTo(0, 10)
            expect(centre.y).toBeCloseTo(0, 10)
            expect(centre.z).toBeCloseTo(0, 10)
        }
    })

    it('keeps both supplied Drifter layouts flat on the N/S axis', () => {
        const outer = buildDrifterOFlat()
        const inner = buildDrifterIFlat()

        expect(outer).toEqual([
            { x: 11272.192, y: 2736.128, z: 0, range: 0.5 },
            { x: -11272.192, y: -2736.128, z: 0, range: 0.5 },
            { x: 0, y: 0, z: 0, range: 32 },
            { x: 0, y: 0, z: 0, range: 32 },
        ])
        expect(inner).toEqual([
            { x: 10682.368, y: 4882.432, z: 0, range: 0.5 },
            { x: -10682.368, y: -4882.432, z: 0, range: 0.5 },
        ])
        expect([...outer, ...inner].every((probe) => probe.z === 0)).toBe(true)
        expect(centroid(outer)).toEqual({ x: 0, y: 0, z: 0 })
        expect(centroid(inner)).toEqual({ x: 0, y: 0, z: 0 })
    })
})
