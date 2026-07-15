import type { Component } from 'vue'
import {
    ArrowLeftRight,
    ArrowUpDown,
    Box,
    ChevronsUpDown,
    CircleDot,
    Crosshair,
    Grid2x2,
    Radar,
} from 'lucide-vue-next'
import type { FormationProbe } from '@/types'
import { AU_KM } from '@/lib/formationEditorUtils'

// Positions use kilometres, scan ranges use AU. Axes follow EVE's solar-system
// convention: +x West, +y Up, +z North.
export const CORE_SCAN_RANGES = [0.25, 0.5, 1, 2, 4, 8, 16, 32] as const
export const COMBAT_SCAN_RANGES = [0.5, 1, 2, 4, 8, 16, 32, 64] as const
export const ALL_SCAN_RANGES = [0.25, 0.5, 1, 2, 4, 8, 16, 32, 64] as const

export type ProbeKind = 'core' | 'combat'
export type BuilderKind =
    | 'pinpoint'
    | 'tetrahedral'
    | 'spread'
    | 'ring'
    | 'shell'
    | 'ladder'
export type FormationAxis = 'northSouth' | 'westEast' | 'upDown'

export function scanRangesFor(kind: ProbeKind): readonly number[] {
    return kind === 'core' ? CORE_SCAN_RANGES : COMBAT_SCAN_RANGES
}

function probe(x: number, y: number, z: number, range: number): FormationProbe {
    return { x, y, z, range }
}

// The familiar EVE-style pinpoint shape: a centre probe surrounded by a
// pentagonal bipyramid. It is kept as the practical client-style baseline,
// rather than being presented as a mathematically proven optimum.
export function buildPinpoint(rangeAu: number): FormationProbe[] {
    const radius = (rangeAu * AU_KM) / 2
    const probes = [probe(0, 0, 0, rangeAu)]
    probes.push(probe(0, radius, 0, rangeAu))
    probes.push(probe(0, -radius, 0, rangeAu))
    for (let i = 0; i < 5; i++) {
        const angle = (i * Math.PI * 2) / 5
        probes.push(
            probe(
                radius * Math.sin(angle),
                0,
                radius * Math.cos(angle),
                rangeAu
            )
        )
    }
    return probes
}

// Experimental geometry-balanced layout. The four closer probes are the vertices
// of a regular tetrahedron; the four outer probes form its inverse. Both sets
// independently have a zero centroid. If EVE uses the four strongest hits, the
// closer set supplies the best-conditioned four-point 3D geometry; the inverse
// set adds symmetric coverage when the estimate is offset from centre.
export function buildTetrahedralScan(rangeAu: number): FormationProbe[] {
    const scanRadiusKm = rangeAu * AU_KM
    const innerRadius = scanRadiusKm * 0.4
    const outerRadius = scanRadiusKm * 0.5
    const tetrahedron: Array<[number, number, number]> = [
        [1, 1, 1],
        [1, -1, -1],
        [-1, 1, -1],
        [-1, -1, 1],
    ]
    const atRadius = ([x, y, z]: [number, number, number], radius: number) => {
        const scale = radius / Math.sqrt(3)
        return probe(x * scale, y * scale, z * scale, rangeAu)
    }
    return [
        ...tetrahedron.map((point) => atRadius(point, innerRadius)),
        ...tetrahedron.map(([x, y, z]) => atRadius([-x, -y, -z], outerRadius)),
    ]
}

// Eight cube corners provide symmetric broad coverage. Each component is half
// the selected scan range, so the formation centre remains inside every sphere.
export function buildSpread(rangeAu: number): FormationProbe[] {
    const component = (rangeAu * AU_KM) / 2
    const probes: FormationProbe[] = []
    for (const x of [-component, component]) {
        for (const y of [-component, component]) {
            for (const z of [-component, component]) {
                probes.push(probe(x, y, z, rangeAu))
            }
        }
    }
    return probes
}

// Eight equally spaced on-grid bookmark points in the horizontal N/S-W/E plane.
export function buildGridRing(
    radiusKm: number,
    rangeAu: number
): FormationProbe[] {
    return Array.from({ length: 8 }, (_, i) => {
        const angle = (i * Math.PI * 2) / 8
        return probe(
            radiusKm * Math.sin(angle),
            0,
            radiusKm * Math.cos(angle),
            rangeAu
        )
    })
}

// Eight octants at an exact radial distance, giving bookmarks above and below
// the grid as well as around it.
export function buildGridShell(
    radiusKm: number,
    rangeAu: number
): FormationProbe[] {
    const component = radiusKm / Math.sqrt(3)
    const probes: FormationProbe[] = []
    for (const x of [-component, component]) {
        for (const y of [-component, component]) {
            for (const z of [-component, component]) {
                probes.push(probe(x, y, z, rangeAu))
            }
        }
    }
    return probes
}

const AXIS_COORDINATES: Record<FormationAxis, 'x' | 'y' | 'z'> = {
    northSouth: 'z',
    westEast: 'x',
    upDown: 'y',
}

// Four mirrored pairs at successive distances along one axis. Custom
// formations are centred by EVE, so every on-grid layout must have zero
// centroid; a one-sided ladder would simply be shifted into a two-sided one.
export function buildDirectionalLadder(
    formationAxis: FormationAxis,
    spacingKm: number,
    rangeAu: number
): FormationProbe[] {
    const axis = AXIS_COORDINATES[formationAxis]
    return [-4, -3, -2, -1, 1, 2, 3, 4].map((step) => {
        const distance = step * spacingKm
        return probe(
            axis === 'x' ? distance : 0,
            axis === 'y' ? distance : 0,
            axis === 'z' ? distance : 0,
            rangeAu
        )
    })
}

// Flattened versions of the two layouts supplied by the user. EVE axes are
// x = West/East, y = Up/Down, z = North/South; z deliberately remains zero.
export function buildDrifterOFlat(): FormationProbe[] {
    return [
        probe(11_272.192, 2_736.128, 0, 0.5),
        probe(-11_272.192, -2_736.128, 0, 0.5),
        probe(0, 0, 0, 32),
        probe(0, 0, 0, 32),
    ]
}

export function buildDrifterIFlat(): FormationProbe[] {
    return [
        probe(10_682.368, 4_882.432, 0, 0.5),
        probe(-10_682.368, -4_882.432, 0, 0.5),
    ]
}

export function buildFormationProbes(options: {
    kind: BuilderKind
    rangeAu: number
    distanceKm: number
    axis: FormationAxis
}): FormationProbe[] {
    switch (options.kind) {
        case 'pinpoint':
            return buildPinpoint(options.rangeAu)
        case 'tetrahedral':
            return buildTetrahedralScan(options.rangeAu)
        case 'spread':
            return buildSpread(options.rangeAu)
        case 'ring':
            return buildGridRing(options.distanceKm, options.rangeAu)
        case 'shell':
            return buildGridShell(options.distanceKm, options.rangeAu)
        case 'ladder':
            return buildDirectionalLadder(
                options.axis,
                options.distanceKm,
                options.rangeAu
            )
    }
}

export interface FormationPreset {
    id: string
    icon: Component
    probes: () => FormationProbe[]
}

function blank(): FormationProbe[] {
    return buildGridShell(250, 0.5)
}

export const SCAN_PRESETS: FormationPreset[] = [
    { id: 'blank', icon: Grid2x2, probes: blank },
    {
        id: 'corePinpoint025',
        icon: Crosshair,
        probes: () => buildPinpoint(0.25),
    },
    {
        id: 'combatPinpoint05',
        icon: Crosshair,
        probes: () => buildPinpoint(0.5),
    },
    {
        id: 'coreTetrahedral025',
        icon: Crosshair,
        probes: () => buildTetrahedralScan(0.25),
    },
    {
        id: 'combatTetrahedral05',
        icon: Crosshair,
        probes: () => buildTetrahedralScan(0.5),
    },
    {
        id: 'systemSpread8',
        icon: Radar,
        probes: () => buildSpread(8),
    },
    {
        id: 'coreDeepSpread32',
        icon: Radar,
        probes: () => buildSpread(32),
    },
    {
        id: 'combatDeepSpread64',
        icon: Radar,
        probes: () => buildSpread(64),
    },
]

export const GRID_PRESETS: FormationPreset[] = [
    {
        id: 'gridRing250',
        icon: CircleDot,
        probes: () => buildGridRing(250, 0.5),
    },
    {
        id: 'gridRing1000',
        icon: CircleDot,
        probes: () => buildGridRing(1000, 0.5),
    },
    {
        id: 'gridShell250',
        icon: Box,
        probes: () => buildGridShell(250, 0.5),
    },
    {
        id: 'gridShell1000',
        icon: Box,
        probes: () => buildGridShell(1000, 0.5),
    },
]

export const SPECIAL_PRESETS: FormationPreset[] = [
    { id: 'drifterOFlat', icon: Crosshair, probes: buildDrifterOFlat },
    { id: 'drifterIFlat', icon: Crosshair, probes: buildDrifterIFlat },
]

const STACK_AXES: Array<{
    id: FormationAxis
    icon: Component
}> = [
    { id: 'northSouth', icon: ArrowUpDown },
    { id: 'westEast', icon: ArrowLeftRight },
    { id: 'upDown', icon: ChevronsUpDown },
]

export const STACK_PRESETS: FormationPreset[] = STACK_AXES.map((axis) => ({
    id: axis.id,
    icon: axis.icon,
    probes: () => buildDirectionalLadder(axis.id, 250, 0.5),
}))
