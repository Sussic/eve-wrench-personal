<script setup lang="ts">
import { computed, ref, watch } from 'vue'
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
import { ArchiveRestore, ShieldCheck } from 'lucide-vue-next'
import type { RecoverySnapshot } from '@/types'
import { useI18n } from '@/composables/useI18n'

const props = defineProps<{
    open: boolean
    snapshots: RecoverySnapshot[]
    busy: boolean
}>()

const emit = defineEmits<{
    cancel: []
    restore: [snapshot: RecoverySnapshot]
}>()

const { t } = useI18n()
const selectedId = ref('')

watch(
    () => [props.open, props.snapshots] as const,
    () => {
        if (
            props.open &&
            !props.snapshots.some((s) => s.id === selectedId.value)
        ) {
            selectedId.value = props.snapshots[0]?.id ?? ''
        }
    },
    { immediate: true }
)

const selected = computed(
    () =>
        props.snapshots.find((snapshot) => snapshot.id === selectedId.value) ??
        null
)

function formatTimestamp(timestamp: number): string {
    return new Intl.DateTimeFormat(undefined, {
        dateStyle: 'medium',
        timeStyle: 'medium',
    }).format(new Date(timestamp * 1000))
}

function formatBytes(bytes: number): string {
    if (bytes < 1024 * 1024)
        return `${Math.max(1, Math.round(bytes / 1024))} KB`
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`
}
</script>

<template>
    <AlertDialog :open="open">
        <AlertDialogContent class="max-w-xl">
            <AlertDialogHeader>
                <AlertDialogTitle>{{
                    t('recovery.restoreTitle')
                }}</AlertDialogTitle>
                <AlertDialogDescription>
                    {{ t('recovery.restoreDescription') }}
                </AlertDialogDescription>
            </AlertDialogHeader>

            <div class="max-h-80 space-y-2 overflow-y-auto pr-1">
                <button
                    v-for="snapshot in snapshots"
                    :key="snapshot.id"
                    type="button"
                    class="flex w-full items-start gap-3 rounded-md border p-3 text-left transition-colors"
                    :class="
                        selectedId === snapshot.id
                            ? 'border-primary bg-primary/5'
                            : 'hover:bg-muted/50'
                    "
                    @click="selectedId = snapshot.id"
                >
                    <ShieldCheck class="mt-0.5 size-4 shrink-0 text-primary" />
                    <div class="min-w-0 flex-1">
                        <div class="flex items-center gap-2">
                            <span class="truncate text-sm font-medium">{{
                                snapshot.label
                            }}</span>
                            <Badge
                                v-if="snapshot === snapshots[0]"
                                variant="secondary"
                            >
                                {{ t('recovery.latestBadge') }}
                            </Badge>
                        </div>
                        <div class="mt-0.5 text-xs text-muted-foreground">
                            {{ formatTimestamp(snapshot.timestamp) }} ·
                            {{
                                t('recovery.snapshotCounts', {
                                    accounts: snapshot.account_count,
                                    characters: snapshot.character_count,
                                    profiles: snapshot.profile_count,
                                })
                            }}
                        </div>
                        <div class="text-[10px] text-muted-foreground/70">
                            {{ snapshot.file_count }} files ·
                            {{ formatBytes(snapshot.size_bytes) }} · v{{
                                snapshot.app_version
                            }}
                        </div>
                    </div>
                </button>
            </div>

            <div
                class="rounded-md border border-amber-500/30 bg-amber-500/10 p-2 text-xs text-amber-800 dark:text-amber-200"
            >
                {{ t('recovery.rescueNote') }}
            </div>

            <AlertDialogFooter>
                <Button
                    variant="outline"
                    :disabled="busy"
                    @click="emit('cancel')"
                >
                    {{ t('common.cancel') }}
                </Button>
                <Button
                    variant="destructive"
                    class="gap-1.5"
                    :disabled="busy || !selected"
                    @click="selected && emit('restore', selected)"
                >
                    <ArchiveRestore class="size-4" />
                    {{
                        busy
                            ? t('recovery.restoring')
                            : t('recovery.restoreSelected')
                    }}
                </Button>
            </AlertDialogFooter>
        </AlertDialogContent>
    </AlertDialog>
</template>
