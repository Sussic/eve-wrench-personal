import type { Component } from 'vue'
import {
    Grid2x2,
    Crosshair,
    Skull,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ChevronsUp,
    ChevronsDown,
} from 'lucide-vue-next'
import type { FormationProbe } from '@/types'

// Preset probe positions are in editor units: km for x/y/z, AU for range.
// Axes follow EVE's solar-system convention: +x West, +y Up, +z North.
const R = 32 // default to the maximum 32 AU scan range

function p(x: number, y: number, z: number): FormationProbe {
    return { x, y, z, range: R }
}

// 8-probe placeholder spread at 250 km — the neutral starting point
function blankSpread(): FormationProbe[] {
    return [
        p(250, 0, 0),
        p(-250, 0, 0),
        p(0, 0, 250),
        p(0, 0, -250),
        p(0, 250, 0),
        p(0, -250, 0),
        p(0, 500, 0),
        p(0, -500, 0),
    ]
}

// Pinpoint: 500 km sphere with an extended 1000 km vertical pair
function pinpoint(): FormationProbe[] {
    return [
        p(500, 0, 0),
        p(-500, 0, 0),
        p(0, 0, 500),
        p(0, 0, -500),
        p(0, 500, 0),
        p(0, -500, 0),
        p(0, 1000, 0),
        p(0, -1000, 0),
    ]
}

// Drifter: one probe placed behind the Drifter (11,500 km West, 3,500 km Up),
// the rest left as 250 km placeholders to reposition
function drifter(): FormationProbe[] {
    return [
        p(11500, 3500, 0),
        p(250, 0, 0),
        p(-250, 0, 0),
        p(0, 0, 250),
        p(0, 0, -250),
        p(0, 250, 0),
        p(0, -250, 0),
        p(0, 500, 0),
    ]
}

export interface FormationPreset {
    id: string // i18n key under formationEditor.presets; 'blank' keeps a generic name
    icon: Component
    probes: () => FormationProbe[]
}

export const FORMATION_PRESETS: FormationPreset[] = [
    { id: 'blank', icon: Grid2x2, probes: blankSpread },
    { id: 'pinpoint', icon: Crosshair, probes: pinpoint },
    { id: 'drifter', icon: Skull, probes: drifter },
]

// Directional stacks: all 8 probes layered along one axis at 200 km intervals
type Stack = {
    id: string
    axis: 'x' | 'y' | 'z'
    sign: 1 | -1
    icon: Component
}
const STACK_DIRS: Stack[] = [
    { id: 'north', axis: 'z', sign: 1, icon: ArrowUp },
    { id: 'south', axis: 'z', sign: -1, icon: ArrowDown },
    { id: 'west', axis: 'x', sign: 1, icon: ArrowLeft }, // +x is West
    { id: 'east', axis: 'x', sign: -1, icon: ArrowRight },
    { id: 'up', axis: 'y', sign: 1, icon: ChevronsUp },
    { id: 'down', axis: 'y', sign: -1, icon: ChevronsDown },
]

function stack(dir: Stack): FormationProbe[] {
    return Array.from({ length: 8 }, (_, i) => {
        const d = (i + 1) * 200 * dir.sign
        return p(
            dir.axis === 'x' ? d : 0,
            dir.axis === 'y' ? d : 0,
            dir.axis === 'z' ? d : 0
        )
    })
}

export const STACK_PRESETS: FormationPreset[] = STACK_DIRS.map((dir) => ({
    id: dir.id,
    icon: dir.icon,
    probes: () => stack(dir),
}))
