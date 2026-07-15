<script setup lang="ts">
import { computed } from 'vue'
import { ArchiveRestore, DatabaseBackup, ShieldCheck } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import type { RecoverySnapshot } from '@/types'
import { useI18n } from '@/composables/useI18n'

const props = defineProps<{
    snapshots: RecoverySnapshot[]
    busy: boolean
    eveRunning: boolean
}>()

const emit = defineEmits<{
    backupAll: []
    restoreAll: []
}>()

const { t } = useI18n()
const latest = computed(() => props.snapshots[0] ?? null)

function formatTimestamp(timestamp: number): string {
    return new Intl.DateTimeFormat(undefined, {
        dateStyle: 'medium',
        timeStyle: 'short',
    }).format(new Date(timestamp * 1000))
}
</script>

<template>
    <section
        class="flex shrink-0 items-center gap-3 border-b bg-muted/30 px-4 py-2"
    >
        <div
            class="flex size-8 shrink-0 items-center justify-center rounded-full bg-primary/10 text-primary"
        >
            <ShieldCheck class="size-4" />
        </div>
        <div class="min-w-0 flex-1">
            <div class="text-xs font-semibold">
                {{ t('recovery.title') }}
            </div>
            <div class="truncate text-[11px] text-muted-foreground">
                <template v-if="latest">
                    {{
                        t('recovery.latest', {
                            time: formatTimestamp(latest.timestamp),
                            accounts: latest.account_count,
                            characters: latest.character_count,
                        })
                    }}
                </template>
                <template v-else>{{ t('recovery.none') }}</template>
            </div>
        </div>
        <Button
            variant="outline"
            size="sm"
            class="h-8 gap-1.5"
            :disabled="busy || eveRunning"
            @click="emit('backupAll')"
        >
            <DatabaseBackup class="size-3.5" />
            {{ t('recovery.backupAll') }}
        </Button>
        <Button
            size="sm"
            class="h-8 gap-1.5"
            :disabled="busy || eveRunning || snapshots.length === 0"
            @click="emit('restoreAll')"
        >
            <ArchiveRestore class="size-3.5" />
            {{ t('recovery.restoreAll') }}
        </Button>
    </section>
</template>
