import type { FormationProbe, ProbeFormation } from '@/types'

export type CoordinateUnit = 'km' | 'au'

export const AU_KM = 149_597_870.7

export type ProbeCompatibility = 'core' | 'combat' | 'both' | 'incompatible'

export function formationCentroid(probes: FormationProbe[]) {
    if (!probes.length) return { x: 0, y: 0, z: 0 }
    const total = probes.reduce(
        (sum, probe) => ({
            x: sum.x + probe.x,
            y: sum.y + probe.y,
            z: sum.z + probe.z,
        }),
        { x: 0, y: 0, z: 0 }
    )
    return {
        x: total.x / probes.length,
        y: total.y / probes.length,
        z: total.z / probes.length,
    }
}

export function centredProbePreview(
    probes: FormationProbe[]
): FormationProbe[] {
    const centroid = formationCentroid(probes)
    return probes.map((probe) => ({
        ...probe,
        x: probe.x - centroid.x,
        y: probe.y - centroid.y,
        z: probe.z - centroid.z,
    }))
}

export function probeCompatibility(
    probes: FormationProbe[]
): ProbeCompatibility {
    const hasCoreOnlyRange = probes.some((probe) => probe.range === 0.25)
    const hasCombatOnlyRange = probes.some((probe) => probe.range === 64)
    if (hasCoreOnlyRange && hasCombatOnlyRange) return 'incompatible'
    if (hasCoreOnlyRange) return 'core'
    if (hasCombatOnlyRange) return 'combat'
    return 'both'
}

export function moveFormationWithIds(
    formations: ProbeFormation[],
    index: number,
    direction: -1 | 1
): boolean {
    const destination = index + direction
    if (index < 0 || destination < 0 || destination >= formations.length) {
        return false
    }

    ;[formations[index], formations[destination]] = [
        formations[destination],
        formations[index],
    ]
    ;[formations[index].id, formations[destination].id] = [
        formations[destination].id,
        formations[index].id,
    ]
    return true
}

export async function deleteFormationAfterConfirmation(
    formations: ProbeFormation[],
    formationId: number,
    confirmDelete: () => Promise<boolean>
): Promise<number | null> {
    const confirmed = await confirmDelete()
    if (!confirmed) return null

    const index = formations.findIndex(
        (formation) => formation.id === formationId
    )
    if (index < 0) return null
    formations.splice(index, 1)
    return index
}

export function coordinateForDisplay(km: number, unit: CoordinateUnit): number {
    if (unit === 'km') return km
    return Number((km / AU_KM).toPrecision(12))
}

export function coordinateFromDisplay(
    value: number,
    unit: CoordinateUnit
): number {
    return unit === 'km' ? value : value * AU_KM
}
