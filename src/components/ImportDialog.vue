<script setup lang="ts">
import { ref, computed } from 'vue'
import {
    AlertDialog,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Switch } from '@/components/ui/switch'
import { ScrollArea } from '@/components/ui/scroll-area'
import { FilePlus, AlertTriangle, CheckCircle } from 'lucide-vue-next'
import type { ImportAnalysis } from '@/types'
import { useI18n } from '@/composables/useI18n'

const { t } = useI18n()

const props = defineProps<{
    open: boolean
    analysis: ImportAnalysis
}>()

const emit = defineEmits<{
    confirm: [overwritePaths: string[]]
    cancel: []
}>()

const selectedConflicts = ref<Set<string>>(
    new Set(props.analysis.conflicts.map((c) => c.relative_path))
)

function toggleConflict(path: string, checked: boolean) {
    const next = new Set(selectedConflicts.value)
    if (checked) {
        next.add(path)
    } else {
        next.delete(path)
    }
    selectedConflicts.value = next
}

function toggleAll(checked: boolean) {
    if (checked) {
        selectedConflicts.value = new Set(
            props.analysis.conflicts.map((c) => c.relative_path)
        )
    } else {
        selectedConflicts.value = new Set()
    }
}

const allSelected = computed(
    () =>
        props.analysis.conflicts.length > 0 &&
        selectedConflicts.value.size === props.analysis.conflicts.length
)

function fileName(relativePath: string): string {
    const parts = relativePath.split('/')
    return parts[parts.length - 1]
}

const serverNames: Record<string, string> = {
    tranquility: 'Tranquility',
    singularity: 'Singularity',
    thunderdome: 'Thunderdome',
    serenity: 'Serenity',
}

function detectServer(segment: string): string | null {
    const lower = segment.toLowerCase()
    for (const [key, label] of Object.entries(serverNames)) {
        if (lower.includes(key)) return label
    }
    return null
}

function parentContext(relativePath: string): string {
    const parts = relativePath.split('/')
    if (parts.length <= 1) return ''

    let server: string | null = null
    let profile: string | null = null

    for (const part of parts) {
        if (!server) server = detectServer(part)
        if (part.startsWith('settings_'))
            profile = part.replace('settings_', '')
    }

    const segments = [server, profile].filter(Boolean)
    return segments.length > 0 ? segments.join(' / ') : ''
}

function handleConfirm() {
    emit('confirm', [...selectedConflicts.value])
}
</script>

<template>
    <AlertDialog :open="open">
        <AlertDialogContent class="max-w-lg">
            <AlertDialogHeader>
                <AlertDialogTitle>{{ t('importDialog.title') }}</AlertDialogTitle>
                <AlertDialogDescription>
                    {{ t('importDialog.foundFiles', { count: analysis.total_files }) }}
                </AlertDialogDescription>
            </AlertDialogHeader>

            <ScrollArea class="max-h-80 pr-3">
                <div class="space-y-4">
                    <!-- New files -->
                    <div v-if="analysis.new_files.length > 0">
                        <div class="mb-1.5 flex items-center gap-2">
                            <FilePlus class="size-4 text-green-500" />
                            <span class="text-sm font-medium">{{ t('importDialog.newFiles') }}</span>
                            <Badge variant="secondary">{{
                                analysis.new_files.length
                            }}</Badge>
                        </div>
                        <ul
                            class="grid grid-cols-[1fr_auto] gap-x-3 gap-y-0.5 pl-6"
                        >
                            <template
                                v-for="f in analysis.new_files"
                                :key="f.relative_path"
                            >
                                <span
                                    class="truncate text-xs font-medium text-foreground"
                                >
                                    {{ fileName(f.relative_path) }}
                                </span>
                                <span
                                    class="text-right text-[10px] text-muted-foreground/60"
                                >
                                    {{ parentContext(f.relative_path) }}
                                </span>
                            </template>
                        </ul>
                    </div>

                    <!-- Conflicts -->
                    <div v-if="analysis.conflicts.length > 0">
                        <div class="mb-1.5 flex items-center gap-2">
                            <AlertTriangle class="size-4 text-yellow-500" />
                            <span class="text-sm font-medium">{{ t('importDialog.conflicts') }}</span>
                            <Badge variant="destructive">{{
                                analysis.conflicts.length
                            }}</Badge>
                            <button
                                class="ml-auto text-xs text-muted-foreground hover:text-foreground transition-colors"
                                @click="toggleAll(!allSelected)"
                            >
                                {{
                                    allSelected ? t('importDialog.deselectAll') : t('importDialog.selectAll')
                                }}
                            </button>
                        </div>
                        <ul
                            class="grid grid-cols-[auto_1fr_auto] items-center gap-x-3 gap-y-1 pl-6"
                        >
                            <template
                                v-for="f in analysis.conflicts"
                                :key="f.relative_path"
                            >
                                <Switch
                                    :model-value="
                                        selectedConflicts.has(f.relative_path)
                                    "
                                    @update:model-value="
                                        toggleConflict(f.relative_path, $event)
                                    "
                                />
                                <span
                                    class="truncate text-xs font-medium text-foreground"
                                >
                                    {{ fileName(f.relative_path) }}
                                </span>
                                <span
                                    class="text-right text-[10px] text-muted-foreground/60"
                                >
                                    {{ parentContext(f.relative_path) }}
                                </span>
                            </template>
                        </ul>
                    </div>

                    <!-- Unchanged -->
                    <div v-if="analysis.unchanged.length > 0">
                        <div class="mb-1.5 flex items-center gap-2">
                            <CheckCircle class="size-4 text-muted-foreground" />
                            <span class="text-sm font-medium">{{ t('importDialog.unchanged') }}</span>
                            <Badge variant="outline">{{
                                analysis.unchanged.length
                            }}</Badge>
                        </div>
                        <ul
                            class="grid grid-cols-[1fr_auto] gap-x-3 gap-y-0.5 pl-6"
                        >
                            <template
                                v-for="f in analysis.unchanged"
                                :key="f.relative_path"
                            >
                                <span
                                    class="truncate text-xs font-medium text-foreground"
                                >
                                    {{ fileName(f.relative_path) }}
                                </span>
                                <span
                                    class="text-right text-[10px] text-muted-foreground/60"
                                >
                                    {{ parentContext(f.relative_path) }}
                                </span>
                            </template>
                        </ul>
                    </div>
                </div>
            </ScrollArea>

            <AlertDialogFooter>
                <Button variant="outline" @click="emit('cancel')">
                    {{ t('common.cancel') }}
                </Button>
                <Button @click="handleConfirm">
                    {{ t('importExport.import') }}
                    <template
                        v-if="
                            analysis.conflicts.length > 0 &&
                            selectedConflicts.size > 0
                        "
                    >
                        ({{ selectedConflicts.size }} {{ t('importDialog.overwrite') }})
                    </template>
                </Button>
            </AlertDialogFooter>
        </AlertDialogContent>
    </AlertDialog>
</template>
