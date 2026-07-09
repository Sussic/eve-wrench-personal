<script setup lang="ts">
import 'vue-sonner/style.css'
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useColorMode } from '@vueuse/core'
import { Toaster } from '@/components/ui/sonner'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Plus, Copy, Trash2, X, Radar } from 'lucide-vue-next'
import { toast } from 'vue-sonner'
import type { FormationProbe, ProbeFormation } from '@/types'
import { shortenPath } from '@/lib/utils'
import { useI18n } from '@/composables/useI18n'

const { t } = useI18n()
const colorMode = useColorMode()

const props = defineProps<{
    filePath: string
    entryName: string
}>()

const KM = 1000
const AU = 149597870700

// Edit state uses km for positions and AU for ranges; meters on the wire
type EditProbe = FormationProbe
type EditFormation = Omit<ProbeFormation, 'id'>

const loading = ref(true)
const saving = ref(false)
const loadError = ref<string | null>(null)
const formations = ref<EditFormation[]>([])
const selected = ref(0)
const scaleFactor = ref(2)
const displayName = ref(props.entryName)
const savedSnapshot = ref('')
const diskChanged = ref(false)
let unlisten: UnlistenFn | null = null

const current = computed<EditFormation | null>(
    () => formations.value[selected.value] ?? null
)

const dirty = computed(
    () => JSON.stringify(formations.value) !== savedSnapshot.value
)

onMounted(async () => {
    await load()
    await refreshDisplayName()
    // The backend broadcasts data-changed to every window on any mutation
    unlisten = await listen('data-changed', onDataChanged)
})

onUnmounted(() => {
    unlisten?.()
})

async function readFormations(): Promise<EditFormation[]> {
    const data = await invoke<ProbeFormation[]>('read_probe_formations', {
        filePath: props.filePath,
    })
    return data.map((f) => ({
        name: f.name,
        probes: f.probes.map((p) => ({
            x: p.x / KM,
            y: p.y / KM,
            z: p.z / KM,
            range: p.range / AU,
        })),
    }))
}

async function load() {
    loading.value = true
    loadError.value = null
    formations.value = []
    selected.value = 0
    diskChanged.value = false
    try {
        formations.value = await readFormations()
        savedSnapshot.value = JSON.stringify(formations.value)
    } catch (e) {
        loadError.value = String(e)
    } finally {
        loading.value = false
    }
}

async function refreshDisplayName() {
    try {
        const name = await invoke<string>('get_entry_display_name', {
            filePath: props.filePath,
        })
        if (name && name !== displayName.value) {
            displayName.value = name
            await getCurrentWindow().setTitle(`Probe Formations — ${name}`)
        }
    } catch {
        // keep the name we have
    }
}

async function onDataChanged() {
    refreshDisplayName()
    let disk: EditFormation[]
    try {
        disk = await readFormations()
    } catch {
        return // file may be mid-write; a later event will catch up
    }
    const diskJson = JSON.stringify(disk)
    if (diskJson === savedSnapshot.value) {
        return // our file wasn't what changed
    }
    if (dirty.value) {
        diskChanged.value = true // don't clobber unsaved edits
        return
    }
    formations.value = disk
    savedSnapshot.value = diskJson
    selected.value = Math.max(
        0,
        Math.min(selected.value, formations.value.length - 1)
    )
}

async function save() {
    saving.value = true
    try {
        const payload: ProbeFormation[] = formations.value.map((f, i) => ({
            id: i,
            name: f.name.trim() || `Formation ${i + 1}`,
            probes: f.probes.map((p) => ({
                x: p.x * KM,
                y: p.y * KM,
                z: p.z * KM,
                range: p.range * AU,
            })),
        }))
        await invoke('write_probe_formations', {
            filePath: props.filePath,
            formations: payload,
        })
        // Normalize edit state to what a re-read would return
        formations.value.forEach((f, i) => {
            f.name = payload[i].name
        })
        savedSnapshot.value = JSON.stringify(formations.value)
        diskChanged.value = false
        toast.success(t('formationEditor.saved'), {
            description: t('formationEditor.savedDesc'),
        })
    } catch (e) {
        toast.error(t('formationEditor.saveFailed'), {
            description: String(e),
        })
    } finally {
        saving.value = false
    }
}

function addFormation() {
    formations.value.push({
        name: `Formation ${formations.value.length + 1}`,
        probes: [
            { x: 250, y: 0, z: 0, range: 0.25 },
            { x: -250, y: 0, z: 0, range: 0.25 },
            { x: 0, y: 0, z: 250, range: 0.25 },
            { x: 0, y: 0, z: -250, range: 0.25 },
            { x: 0, y: 250, z: 0, range: 0.25 },
            { x: 0, y: -250, z: 0, range: 0.25 },
            { x: 0, y: 500, z: 0, range: 0.25 },
            { x: 0, y: -500, z: 0, range: 0.25 },
        ],
    })
    selected.value = formations.value.length - 1
}

function duplicateFormation() {
    if (!current.value) return
    formations.value.push({
        name: `${current.value.name} copy`,
        probes: current.value.probes.map((p) => ({ ...p })),
    })
    selected.value = formations.value.length - 1
}

function deleteFormation() {
    if (!current.value) return
    formations.value.splice(selected.value, 1)
    selected.value = Math.min(selected.value, formations.value.length - 1)
}

// EVE probe launchers hold 8 probes, so formations are capped at 8
function addProbe() {
    if (!current.value || current.value.probes.length >= 8) return
    current.value.probes.push({ x: 0, y: 0, z: 0, range: 0.25 })
}

function removeProbe(index: number) {
    current.value?.probes.splice(index, 1)
}

function applyScale() {
    if (!current.value || !isFinite(scaleFactor.value)) return
    for (const p of current.value.probes) {
        p.x *= scaleFactor.value
        p.y *= scaleFactor.value
        p.z *= scaleFactor.value
    }
}

function updateProbe(probe: EditProbe, key: keyof EditProbe, value: unknown) {
    const num = Number(value)
    if (isFinite(num)) probe[key] = num
}

// Valid probe scan ranges in EVE: powers of two from 0.25 to 32 AU
const RANGE_OPTIONS = [0.25, 0.5, 1, 2, 4, 8, 16, 32]

function rangeOptionsFor(value: number): number[] {
    // Keep unusual values from existing files selectable instead of lying
    return RANGE_OPTIONS.includes(value)
        ? RANGE_OPTIONS
        : [...RANGE_OPTIONS, value].sort((a, b) => a - b)
}

function setAllRanges(value: unknown) {
    const num = Number(value)
    if (!current.value || !isFinite(num)) return
    for (const p of current.value.probes) {
        p.range = num
    }
}

// ── Preview projection ───────────────────────────────────────────────────

const yaw = ref(0.6)
const pitch = ref(0.4)
const zoom = ref(1)
let dragging = false
let lastX = 0
let lastY = 0

function setTextSelection(enabled: boolean) {
    const value = enabled ? '' : 'none'
    document.body.style.userSelect = value
    document.body.style.webkitUserSelect = value
}

function onPointerDown(e: PointerEvent) {
    dragging = true
    lastX = e.clientX
    lastY = e.clientY
    setTextSelection(false)
    ;(e.target as Element).setPointerCapture(e.pointerId)
}

function onPointerMove(e: PointerEvent) {
    if (!dragging) return
    yaw.value += (e.clientX - lastX) * 0.01
    pitch.value = Math.max(
        -Math.PI / 2,
        Math.min(Math.PI / 2, pitch.value + (e.clientY - lastY) * 0.01)
    )
    lastX = e.clientX
    lastY = e.clientY
}

function onPointerUp() {
    dragging = false
    setTextSelection(true)
}

function onWheel(e: WheelEvent) {
    zoom.value = Math.max(
        0.2,
        Math.min(4, zoom.value * (e.deltaY < 0 ? 1.1 : 0.9))
    )
}

function project(x: number, y: number, z: number) {
    const x1 = x * Math.cos(yaw.value) + z * Math.sin(yaw.value)
    const z1 = -x * Math.sin(yaw.value) + z * Math.cos(yaw.value)
    const y1 = y * Math.cos(pitch.value) - z1 * Math.sin(pitch.value)
    const depth = y * Math.sin(pitch.value) + z1 * Math.cos(pitch.value)
    return { sx: x1, sy: -y1, depth }
}

const extent = computed(() => {
    if (!current.value) return 1
    let max = 1
    for (const p of current.value.probes) {
        max = Math.max(max, Math.abs(p.x), Math.abs(p.y), Math.abs(p.z))
    }
    return max
})

const previewScale = computed(() => (160 / extent.value) * zoom.value)

const axes = computed(() => {
    const e = extent.value
    return (
        [
            { label: 'X', x: e, y: 0, z: 0, color: '#f85149' },
            { label: 'Y', x: 0, y: e, z: 0, color: '#3fb950' },
            { label: 'Z', x: 0, y: 0, z: e, color: '#58a6ff' },
        ] as const
    ).map((axis) => {
        const from = project(-axis.x, -axis.y, -axis.z)
        const to = project(axis.x, axis.y, axis.z)
        const s = previewScale.value
        return {
            label: axis.label,
            color: axis.color,
            x1: from.sx * s,
            y1: from.sy * s,
            x2: to.sx * s,
            y2: to.sy * s,
        }
    })
})

// EVE solarsystem coordinates (right-handed): +Z is North, +Y is Up, +X is West
const compass = computed(() => {
    const e = extent.value * 1.14
    const s = previewScale.value
    return [
        { label: 'N', x: 0, y: 0, z: e, emphasis: true },
        { label: 'S', x: 0, y: 0, z: -e, emphasis: false },
        { label: 'W', x: e, y: 0, z: 0, emphasis: false },
        { label: 'E', x: -e, y: 0, z: 0, emphasis: false },
        { label: '↑', x: 0, y: e, z: 0, emphasis: false },
    ].map((m) => {
        const p = project(m.x, m.y, m.z)
        return { ...m, sx: p.sx * s, sy: p.sy * s }
    })
})

// Round ring spacing to 1/2/2.5/5 × 10^n so the labels are nice numbers
function niceStep(target: number): number {
    const pow = Math.pow(10, Math.floor(Math.log10(Math.max(1, target))))
    for (const mult of [1, 2, 2.5, 5, 10]) {
        if (pow * mult >= target) return pow * mult
    }
    return pow * 10
}

function formatDistance(km: number): string {
    const AU_KM = 149597870.7
    if (km >= AU_KM / 100) return `${(km / AU_KM).toFixed(2)} AU`
    return `${Math.round(km).toLocaleString()} km`
}

// Holographic scale rings in the horizontal (XZ) plane
const rings = computed(() => {
    const step = niceStep(extent.value / 3)
    const s = previewScale.value
    const SEGMENTS = 72
    return [step, step * 2, step * 3].map((radius) => {
        const points: string[] = []
        for (let i = 0; i < SEGMENTS; i++) {
            const angle = (i / SEGMENTS) * Math.PI * 2
            const p = project(
                radius * Math.cos(angle),
                0,
                radius * Math.sin(angle)
            )
            points.push(`${(p.sx * s).toFixed(1)},${(p.sy * s).toFixed(1)}`)
        }
        // Label sits on the ring's south-west arc, usually facing the viewer
        const labelPos = project(
            radius * Math.SQRT1_2,
            0,
            -radius * Math.SQRT1_2
        )
        return {
            radius,
            points: points.join(' '),
            labelX: labelPos.sx * s,
            labelY: labelPos.sy * s,
            label: formatDistance(radius),
        }
    })
})

const projectedProbes = computed(() => {
    if (!current.value) return []
    const s = previewScale.value
    return current.value.probes
        .map((p, index) => {
            const { sx, sy, depth } = project(p.x, p.y, p.z)
            return { index, x: sx * s, y: sy * s, depth, probe: p }
        })
        .sort((a, b) => a.depth - b.depth)
})
</script>

<template>
    <div
        class="fixed inset-0 flex flex-col overflow-hidden bg-background text-foreground"
        :class="colorMode"
    >
        <Toaster
            position="top-center"
            rich-colors
            :theme="colorMode === 'dark' ? 'dark' : 'light'"
        />

        <header class="flex items-center gap-3 border-b px-4 py-3">
            <Radar class="size-5 shrink-0 text-muted-foreground" />
            <div class="min-w-0 flex-1">
                <h1 class="truncate text-sm font-semibold">
                    {{ t('formationEditor.title', { name: displayName }) }}
                </h1>
                <p
                    class="truncate text-xs text-muted-foreground"
                    :title="filePath"
                >
                    {{ shortenPath(filePath) }}
                </p>
            </div>
            <Button
                size="sm"
                :disabled="saving || loading || !dirty"
                @click="save"
            >
                {{ t('formationEditor.save') }}
            </Button>
        </header>

        <div
            v-if="diskChanged"
            class="flex items-center justify-between gap-2 border-b bg-amber-500/10 px-4 py-1.5"
        >
            <span class="text-xs text-amber-600 dark:text-amber-400">
                {{ t('formationEditor.fileChanged') }}
            </span>
            <Button variant="outline" size="sm" class="h-6" @click="load">
                {{ t('formationEditor.reload') }}
            </Button>
        </div>

        <main
            v-if="loading"
            class="flex flex-1 items-center justify-center text-muted-foreground"
        >
            {{ t('formationEditor.loading') }}
        </main>

        <main
            v-else-if="loadError"
            class="flex flex-1 flex-col items-center justify-center gap-3 p-8 text-center"
        >
            <p class="font-medium">{{ t('formationEditor.loadFailed') }}</p>
            <p class="max-w-lg text-sm text-muted-foreground">
                {{ loadError }}
            </p>
            <Button variant="outline" size="sm" @click="load">
                {{ t('common.refresh') }}
            </Button>
        </main>

        <main v-else class="flex min-h-0 flex-1">
            <!-- Left: control column -->
            <aside
                class="flex w-[440px] shrink-0 flex-col border-r bg-muted/20"
            >
                <!-- Formations -->
                <section class="border-b px-4 pb-3 pt-4">
                    <div class="mb-2 flex items-center justify-between">
                        <span
                            class="text-[10px] font-medium uppercase tracking-[0.18em] text-muted-foreground"
                        >
                            {{ t('formationEditor.formations') }}
                        </span>
                        <div class="flex items-center gap-0.5">
                            <Button
                                variant="ghost"
                                size="icon"
                                class="size-6"
                                :title="t('formationEditor.addFormation')"
                                @click="addFormation"
                            >
                                <Plus class="size-3.5" />
                            </Button>
                            <Button
                                v-if="current"
                                variant="ghost"
                                size="icon"
                                class="size-6"
                                :title="t('formationEditor.duplicate')"
                                @click="duplicateFormation"
                            >
                                <Copy class="size-3.5" />
                            </Button>
                            <Button
                                v-if="current"
                                variant="ghost"
                                size="icon"
                                class="size-6 hover:text-destructive"
                                :title="t('formationEditor.deleteFormation')"
                                @click="deleteFormation"
                            >
                                <Trash2 class="size-3.5" />
                            </Button>
                        </div>
                    </div>
                    <div
                        v-if="!formations.length"
                        class="py-4 text-center text-sm text-muted-foreground"
                    >
                        {{ t('formationEditor.noFormations') }}
                    </div>
                    <div v-else class="flex flex-col gap-px">
                        <div
                            v-for="(f, i) in formations"
                            :key="i"
                            class="flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 transition-colors"
                            :class="
                                i === selected
                                    ? 'bg-background shadow-sm'
                                    : 'hover:bg-background/50'
                            "
                            @click="selected = i"
                        >
                            <span
                                class="h-4 w-0.5 shrink-0 rounded-full transition-colors"
                                :class="
                                    i === selected
                                        ? 'bg-cyan-400'
                                        : 'bg-transparent'
                                "
                            />
                            <input
                                v-model="f.name"
                                class="w-full min-w-0 bg-transparent text-sm outline-none"
                                :class="
                                    i === selected
                                        ? 'font-medium'
                                        : 'text-muted-foreground'
                                "
                                :placeholder="t('formationEditor.name')"
                                @focus="selected = i"
                            />
                            <span
                                class="shrink-0 font-mono text-[10px] tabular-nums text-muted-foreground"
                            >
                                {{ f.probes.length }}/8
                            </span>
                        </div>
                    </div>
                </section>

                <!-- Probes -->
                <section
                    v-if="current"
                    class="flex min-h-0 flex-1 flex-col px-4 pb-2 pt-3"
                >
                    <div class="mb-2 flex items-center justify-between">
                        <span
                            class="text-[10px] font-medium uppercase tracking-[0.18em] text-muted-foreground"
                        >
                            {{ t('formationEditor.probes') }}
                        </span>
                        <Button
                            variant="ghost"
                            size="icon"
                            class="size-6"
                            :disabled="current.probes.length >= 8"
                            :title="
                                current.probes.length >= 8
                                    ? t('formationEditor.maxProbes')
                                    : t('formationEditor.addProbe')
                            "
                            @click="addProbe"
                        >
                            <Plus class="size-3.5" />
                        </Button>
                    </div>
                    <div
                        class="grid grid-cols-[1.25rem_1fr_1fr_1fr_4.5rem_1.5rem] items-center gap-1 pb-1 text-[10px] uppercase tracking-wider text-muted-foreground"
                    >
                        <span></span>
                        <span class="text-right">X</span>
                        <span class="text-right">Y</span>
                        <span class="text-right">Z</span>
                        <span class="text-center">AU</span>
                        <span></span>
                    </div>
                    <ScrollArea class="min-h-0 flex-1 pr-1">
                        <div class="flex flex-col gap-1">
                            <div
                                v-for="(p, i) in current.probes"
                                :key="i"
                                class="group grid grid-cols-[1.25rem_1fr_1fr_1fr_4.5rem_1.5rem] items-center gap-1"
                            >
                                <span
                                    class="font-mono text-[10px] tabular-nums text-muted-foreground"
                                >
                                    {{ String(i + 1).padStart(2, '0') }}
                                </span>
                                <Input
                                    type="number"
                                    step="50"
                                    class="h-7 border-input/50 bg-background/60 px-1.5 text-right font-mono text-xs tabular-nums"
                                    :model-value="p.x"
                                    @update:model-value="
                                        updateProbe(p, 'x', $event)
                                    "
                                />
                                <Input
                                    type="number"
                                    step="50"
                                    class="h-7 border-input/50 bg-background/60 px-1.5 text-right font-mono text-xs tabular-nums"
                                    :model-value="p.y"
                                    @update:model-value="
                                        updateProbe(p, 'y', $event)
                                    "
                                />
                                <Input
                                    type="number"
                                    step="50"
                                    class="h-7 border-input/50 bg-background/60 px-1.5 text-right font-mono text-xs tabular-nums"
                                    :model-value="p.z"
                                    @update:model-value="
                                        updateProbe(p, 'z', $event)
                                    "
                                />
                                <select
                                    :value="p.range"
                                    class="h-7 rounded-md border border-input/50 bg-background/60 px-1 text-center font-mono text-xs"
                                    @change="
                                        updateProbe(
                                            p,
                                            'range',
                                            ($event.target as HTMLSelectElement)
                                                .value
                                        )
                                    "
                                >
                                    <option
                                        v-for="r in rangeOptionsFor(p.range)"
                                        :key="r"
                                        :value="r"
                                    >
                                        {{ r }}
                                    </option>
                                </select>
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    class="size-6 opacity-0 transition-opacity hover:text-destructive group-hover:opacity-100"
                                    @click="removeProbe(i)"
                                >
                                    <X class="size-3" />
                                </Button>
                            </div>
                        </div>
                    </ScrollArea>
                </section>

                <!-- Tools -->
                <section
                    v-if="current"
                    class="flex items-center gap-3 border-t px-4 py-2.5"
                >
                    <label class="flex items-center gap-1.5">
                        <span
                            class="text-[10px] uppercase tracking-wider text-muted-foreground"
                        >
                            {{ t('formationEditor.allRanges') }}
                        </span>
                        <select
                            value=""
                            class="h-7 rounded-md border border-input/50 bg-background/60 px-1.5 font-mono text-xs"
                            @change="
                                setAllRanges(
                                    ($event.target as HTMLSelectElement).value
                                )
                                ;($event.target as HTMLSelectElement).value = ''
                            "
                        >
                            <option value="" disabled>—</option>
                            <option
                                v-for="r in RANGE_OPTIONS"
                                :key="r"
                                :value="r"
                            >
                                {{ r }} AU
                            </option>
                        </select>
                    </label>
                    <label class="ml-auto flex items-center gap-1.5">
                        <span
                            class="text-[10px] uppercase tracking-wider text-muted-foreground"
                        >
                            {{ t('formationEditor.scale') }}
                        </span>
                        <Input
                            type="number"
                            step="0.5"
                            class="h-7 w-16 border-input/50 bg-background/60 px-1.5 text-right font-mono text-xs tabular-nums"
                            :model-value="scaleFactor"
                            @update:model-value="scaleFactor = Number($event)"
                        />
                    </label>
                    <Button
                        variant="outline"
                        size="sm"
                        class="h-7"
                        @click="applyScale"
                    >
                        {{ t('formationEditor.apply') }}
                    </Button>
                </section>
            </aside>

            <!-- Right: full-bleed preview viewport -->
            <div class="relative min-w-0 flex-1">
                <svg
                    viewBox="-200 -200 400 400"
                    class="absolute inset-0 h-full w-full cursor-grab touch-none active:cursor-grabbing"
                    @pointerdown="onPointerDown"
                    @pointermove="onPointerMove"
                    @pointerup="onPointerUp"
                    @pointercancel="onPointerUp"
                    @wheel.prevent="onWheel"
                >
                    <line
                        v-for="axis in axes"
                        :key="axis.label"
                        :x1="axis.x1"
                        :y1="axis.y1"
                        :x2="axis.x2"
                        :y2="axis.y2"
                        :stroke="axis.color"
                        stroke-width="1"
                        opacity="0.5"
                    />
                    <g stroke="#22d3ee" fill="none">
                        <polygon
                            v-for="ring in rings"
                            :key="`ring-${ring.radius}`"
                            :points="ring.points"
                            stroke-width="1"
                            stroke-dasharray="5 4"
                            opacity="0.25"
                        />
                    </g>
                    <text
                        v-for="ring in rings"
                        :key="`ring-label-${ring.radius}`"
                        :x="ring.labelX"
                        :y="ring.labelY"
                        fill="#22d3ee"
                        font-size="7"
                        opacity="0.7"
                        dx="3"
                        dy="-2"
                    >
                        {{ ring.label }}
                    </text>
                    <text
                        v-for="mark in compass"
                        :key="`compass-${mark.label}`"
                        :x="mark.sx"
                        :y="mark.sy"
                        :font-size="mark.emphasis ? 13 : 9"
                        :font-weight="mark.emphasis ? 'bold' : 'normal'"
                        :fill="mark.emphasis ? '#22d3ee' : 'currentColor'"
                        :opacity="mark.emphasis ? 1 : 0.5"
                        class="text-muted-foreground"
                        text-anchor="middle"
                        dominant-baseline="middle"
                    >
                        {{ mark.label }}
                    </text>
                    <circle cx="0" cy="0" r="2" class="fill-muted-foreground" />
                    <circle
                        v-for="p in projectedProbes"
                        :key="p.index"
                        :cx="p.x"
                        :cy="p.y"
                        :r="4 + p.depth / extent"
                        class="fill-primary stroke-background"
                        stroke-width="1"
                    >
                        <title>
                            #{{ p.index + 1 }}: ({{ p.probe.x }},
                            {{ p.probe.y }}, {{ p.probe.z }}) km —
                            {{ p.probe.range }} AU
                        </title>
                    </circle>
                </svg>
                <div
                    class="pointer-events-none absolute inset-x-0 bottom-3 flex justify-center"
                >
                    <span
                        class="rounded-full border bg-background/70 px-3 py-1 text-[11px] text-muted-foreground backdrop-blur"
                    >
                        {{ t('formationEditor.previewHint') }}
                    </span>
                </div>
            </div>
        </main>
    </div>
</template>

<style scoped>
/* Arrow keys still step; the spinner chrome fights the compact grid */
input[type='number']::-webkit-outer-spin-button,
input[type='number']::-webkit-inner-spin-button {
    -webkit-appearance: none;
    margin: 0;
}
input[type='number'] {
    -moz-appearance: textfield;
    appearance: textfield;
}
</style>
