<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
    AlertDialog,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { Dices, LoaderCircle } from 'lucide-vue-next'
import type {
    FormationSnapshot,
    FormationVariationAxes,
    ProbeFormation,
    SettingsEntry,
} from '@/types'
import { useI18n } from '@/composables/useI18n'

const props = defineProps<{
    open: boolean
    source: SettingsEntry | null
    targets: SettingsEntry[]
    busy: boolean
}>()

const emit = defineEmits<{
    cancel: []
    apply: [
        options: {
            variedFormationIds: number[]
            variabilityKm: number
            axes: FormationVariationAxes
        },
    ]
}>()

const { t } = useI18n()
const loading = ref(false)
const loadError = ref('')
const formations = ref<ProbeFormation[]>([])
const selectedIds = ref<Set<number>>(new Set())
const variabilityKm = ref(40)
const axes = ref<FormationVariationAxes>({
    northSouth: true,
    westEast: true,
    upDown: true,
})

function likelyOnGrid(formation: ProbeFormation): boolean {
    const fiveThousandKm = 5_000_000
    return (
        formation.probes.length > 0 &&
        formation.probes.every((probe) =>
            [probe.x, probe.y, probe.z].every(
                (coordinate) => Math.abs(coordinate) <= fiveThousandKm
            )
        )
    )
}

async function loadFormations() {
    if (!props.source) return
    loading.value = true
    loadError.value = ''
    try {
        const snapshot = await invoke<FormationSnapshot>(
            'read_probe_formations',
            { filePath: props.source.path }
        )
        formations.value = snapshot.formations
        selectedIds.value = new Set(
            snapshot.formations
                .filter(likelyOnGrid)
                .map((formation) => formation.id)
        )
    } catch (error) {
        loadError.value = String(error)
        formations.value = []
        selectedIds.value = new Set()
    } finally {
        loading.value = false
    }
}

watch(
    () => props.open,
    (open) => {
        if (open) void loadFormations()
    }
)

const valid = computed(
    () =>
        !loading.value &&
        !loadError.value &&
        selectedIds.value.size > 0 &&
        Number.isFinite(Number(variabilityKm.value)) &&
        Number(variabilityKm.value) > 0 &&
        Number(variabilityKm.value) <= 100_000 &&
        Object.values(axes.value).some(Boolean)
)

function toggleFormation(id: number, checked: boolean) {
    const next = new Set(selectedIds.value)
    if (checked) next.add(id)
    else next.delete(id)
    selectedIds.value = next
}

function selectAll(value: boolean) {
    selectedIds.value = value
        ? new Set(formations.value.map((formation) => formation.id))
        : new Set()
}

function setAxis(axis: keyof FormationVariationAxes, checked: boolean) {
    axes.value = { ...axes.value, [axis]: checked }
}

function apply() {
    if (!valid.value) return
    emit('apply', {
        variedFormationIds: [...selectedIds.value],
        variabilityKm: Number(variabilityKm.value),
        axes: axes.value,
    })
}
</script>

<template>
    <AlertDialog :open="open">
        <AlertDialogContent
            class="max-h-[calc(100vh-2rem)] overflow-y-auto"
            style="width: calc(100vw - 2rem); max-width: 48rem"
        >
            <AlertDialogHeader>
                <AlertDialogTitle>{{
                    t('formationVariants.title')
                }}</AlertDialogTitle>
                <AlertDialogDescription>
                    {{
                        t('formationVariants.description', {
                            source: source?.display_name ?? '',
                            targets: targets.length,
                        })
                    }}
                </AlertDialogDescription>
            </AlertDialogHeader>

            <div
                v-if="loading"
                class="flex h-32 items-center justify-center text-muted-foreground"
            >
                <LoaderCircle class="mr-2 size-4 animate-spin" />
                {{ t('common.loading') }}
            </div>
            <div
                v-else-if="loadError"
                class="rounded-md border border-destructive/40 bg-destructive/5 p-3 text-sm text-destructive"
            >
                {{ loadError }}
            </div>
            <template v-else>
                <div
                    class="grid min-w-0 grid-cols-1 gap-4 md:grid-cols-[minmax(0,1fr)_190px]"
                >
                    <div>
                        <div class="mb-2 flex items-center gap-2">
                            <span class="text-sm font-medium">{{
                                t('formationVariants.choose')
                            }}</span>
                            <Badge variant="secondary"
                                >{{ selectedIds.size }}/{{
                                    formations.length
                                }}</Badge
                            >
                            <Button
                                variant="ghost"
                                size="sm"
                                class="ml-auto h-6 px-2 text-xs"
                                @click="selectAll(true)"
                            >
                                {{ t('common.all') }}
                            </Button>
                            <Button
                                variant="ghost"
                                size="sm"
                                class="h-6 px-2 text-xs"
                                @click="selectAll(false)"
                            >
                                {{ t('common.none') }}
                            </Button>
                        </div>
                        <div
                            class="max-h-56 divide-y overflow-y-auto rounded-md border"
                        >
                            <label
                                v-for="formation in formations"
                                :key="formation.id"
                                class="flex cursor-pointer items-center gap-2 px-3 py-2 text-sm"
                            >
                                <Checkbox
                                    :model-value="selectedIds.has(formation.id)"
                                    @update:model-value="
                                        toggleFormation(
                                            formation.id,
                                            $event === true
                                        )
                                    "
                                />
                                <span class="min-w-0 flex-1 truncate">{{
                                    formation.name
                                }}</span>
                                <span class="text-[10px] text-muted-foreground">
                                    {{
                                        selectedIds.has(formation.id)
                                            ? t('formationVariants.varied')
                                            : t('formationVariants.exact')
                                    }}
                                </span>
                            </label>
                            <div
                                v-if="formations.length === 0"
                                class="p-6 text-center text-sm text-muted-foreground"
                            >
                                {{ t('formationVariants.empty') }}
                            </div>
                        </div>
                    </div>

                    <div class="space-y-4 rounded-md border bg-muted/20 p-3">
                        <label class="block text-xs font-medium">
                            {{ t('formationVariants.maximum') }}
                            <div class="mt-1 flex items-center gap-2">
                                <Input
                                    v-model.number="variabilityKm"
                                    type="number"
                                    min="0.001"
                                    max="100000"
                                    step="1"
                                />
                                <span class="text-xs text-muted-foreground"
                                    >km</span
                                >
                            </div>
                        </label>
                        <div>
                            <div class="mb-1.5 text-xs font-medium">
                                {{ t('formationVariants.axes') }}
                            </div>
                            <div class="space-y-2 text-xs">
                                <label class="flex items-center gap-2">
                                    <Checkbox
                                        :model-value="axes.northSouth"
                                        @update:model-value="
                                            setAxis(
                                                'northSouth',
                                                $event === true
                                            )
                                        "
                                    />
                                    {{ t('formationVariants.northSouth') }}
                                </label>
                                <label class="flex items-center gap-2">
                                    <Checkbox
                                        :model-value="axes.westEast"
                                        @update:model-value="
                                            setAxis('westEast', $event === true)
                                        "
                                    />
                                    {{ t('formationVariants.westEast') }}
                                </label>
                                <label class="flex items-center gap-2">
                                    <Checkbox
                                        :model-value="axes.upDown"
                                        @update:model-value="
                                            setAxis('upDown', $event === true)
                                        "
                                    />
                                    {{ t('formationVariants.upDown') }}
                                </label>
                            </div>
                        </div>
                    </div>
                </div>

                <div
                    class="rounded-md border border-primary/20 bg-primary/5 p-2 text-xs text-muted-foreground"
                >
                    {{ t('formationVariants.stableNote') }}
                </div>
            </template>

            <AlertDialogFooter class="flex-wrap gap-2">
                <Button
                    variant="outline"
                    :disabled="busy"
                    @click="emit('cancel')"
                >
                    {{ t('common.cancel') }}
                </Button>
                <Button
                    class="gap-1.5"
                    :disabled="busy || !valid"
                    @click="apply"
                >
                    <Dices class="size-4" />
                    {{
                        busy
                            ? t('formationVariants.applying')
                            : t('formationVariants.apply')
                    }}
                </Button>
            </AlertDialogFooter>
        </AlertDialogContent>
    </AlertDialog>
</template>
