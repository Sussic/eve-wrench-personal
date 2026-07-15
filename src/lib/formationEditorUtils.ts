import type { ProbeFormation } from '@/types'

export type CoordinateUnit = 'km' | 'au'

export const AU_KM = 149_597_870.7

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
