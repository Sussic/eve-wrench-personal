<script setup lang="ts">
import { computed } from 'vue'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Checkbox } from '@/components/ui/checkbox'
import { User, Rocket, ArrowDown, X, Copy } from 'lucide-vue-next'
import type { SourceItem, SettingsEntry } from '@/types'
import { isBackup, getServerShortName, getServerColor } from '@/types'
import { groupsForKind } from '@/lib/copyGroups'
import { useI18n } from '@/composables/useI18n'

const props = defineProps<{
    source: SourceItem | null
    targets: SettingsEntry[]
    canCopy: boolean
    copying: boolean
    groupSelection: Record<string, boolean>
}>()

const emit = defineEmits<{
    clearSource: []
    removeTarget: [entry: SettingsEntry]
    clearTargets: []
    executeCopy: []
    setGroup: [id: string, value: boolean]
}>()

const { t } = useI18n()

const visibleGroups = computed(() =>
    props.source ? groupsForKind(props.source.kind) : []
)
</script>

<template>
    <aside
        class="flex w-72 shrink-0 flex-col gap-3 overflow-hidden border-l bg-muted/20 p-4"
    >
        <div class="shrink-0">
            <div class="mb-1 flex items-center justify-between">
                <span class="text-xs font-medium text-muted-foreground">{{
                    t('copyPanel.source')
                }}</span>
                <Button
                    v-if="source"
                    variant="ghost"
                    size="sm"
                    class="h-5 px-1.5 text-xs"
                    @click="emit('clearSource')"
                >
                    {{ t('common.clear') }}
                </Button>
            </div>
            <div
                v-if="source"
                class="flex items-center gap-2 rounded border bg-background p-1.5"
            >
                <div
                    class="flex size-6 shrink-0 items-center justify-center overflow-hidden rounded"
                >
                    <img
                        v-if="!isBackup(source) && source.character"
                        :src="source.character.portrait_url"
                        class="size-full object-cover"
                        @error="
                            ($event.target as HTMLImageElement).style.display =
                                'none'
                        "
                    />
                    <Rocket
                        v-else-if="source.kind === 'char'"
                        class="size-3 text-muted-foreground"
                    />
                    <User v-else class="size-3 text-muted-foreground" />
                </div>
                <div class="flex min-w-0 flex-1 flex-col">
                    <span class="truncate text-xs font-medium">{{
                        source.display_name
                    }}</span>
                    <span
                        v-if="isBackup(source)"
                        class="text-[10px] text-muted-foreground"
                        >{{ t('titleBar.backups') }}</span
                    >
                </div>
                <Badge
                    v-if="!isBackup(source)"
                    variant="outline"
                    class="shrink-0 px-1.5 py-0 text-[10px]"
                    :style="{
                        borderColor: getServerColor(source.server),
                        color: getServerColor(source.server),
                    }"
                >
                    {{ getServerShortName(source.server) }}
                </Badge>
            </div>
            <div
                v-else
                class="rounded border border-dashed bg-background/50 p-3 text-center text-xs text-muted-foreground"
            >
                {{ t('copyPanel.noSourceSelected') }}
            </div>
        </div>

        <div class="flex shrink-0 justify-center py-0.5 text-muted-foreground">
            <ArrowDown class="size-4" />
        </div>

        <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
            <div class="mb-1 flex shrink-0 items-center justify-between">
                <span class="text-xs font-medium text-muted-foreground">
                    {{ t('copyPanel.targets') }}
                    <span v-if="targets.length" class="ml-1 text-foreground"
                        >({{ targets.length }})</span
                    >
                </span>
                <Button
                    v-if="targets.length"
                    variant="ghost"
                    size="sm"
                    class="h-5 px-1.5 text-xs"
                    @click="emit('clearTargets')"
                >
                    {{ t('common.clear') }}
                </Button>
            </div>
            <div class="flex-1 overflow-y-auto rounded border bg-background">
                <div v-if="targets.length" class="divide-y">
                    <div
                        v-for="target in targets"
                        :key="target.path"
                        class="group flex items-center gap-2 p-1.5"
                    >
                        <div
                            class="flex size-5 shrink-0 items-center justify-center overflow-hidden rounded"
                        >
                            <img
                                v-if="target.character"
                                :src="target.character.portrait_url"
                                class="size-full object-cover"
                                @error="
                                    (
                                        $event.target as HTMLImageElement
                                    ).style.display = 'none'
                                "
                            />
                            <Rocket
                                v-else-if="target.kind === 'char'"
                                class="size-2.5 text-muted-foreground"
                            />
                            <User
                                v-else
                                class="size-2.5 text-muted-foreground"
                            />
                        </div>
                        <span class="min-w-0 flex-1 truncate text-xs">{{
                            target.display_name
                        }}</span>
                        <Badge
                            variant="outline"
                            class="shrink-0 px-1 py-0 text-[10px]"
                            :style="{
                                borderColor: getServerColor(target.server),
                                color: getServerColor(target.server),
                            }"
                        >
                            {{ getServerShortName(target.server) }}
                        </Badge>
                        <Button
                            variant="ghost"
                            size="icon"
                            class="size-5 opacity-0 transition-opacity group-hover:opacity-100"
                            @click="emit('removeTarget', target)"
                        >
                            <X class="size-3" />
                        </Button>
                    </div>
                </div>
                <div
                    v-else
                    class="flex h-full min-h-20 items-center justify-center p-3 text-center text-xs text-muted-foreground"
                >
                    {{ t('copyPanel.noTargetsSelected') }}
                </div>
            </div>
        </div>

        <div v-if="source" class="shrink-0">
            <span class="mb-1 block text-xs font-medium text-muted-foreground">
                {{ t('copyPanel.copyOptions') }}
            </span>
            <div
                class="flex max-h-48 flex-col gap-1.5 overflow-y-auto rounded border bg-background p-2"
            >
                <label
                    v-for="group in visibleGroups"
                    :key="group.id"
                    class="flex cursor-pointer items-center gap-2 text-xs"
                >
                    <Checkbox
                        :model-value="groupSelection[group.id]"
                        @update:model-value="
                            emit('setGroup', group.id, $event === true)
                        "
                    />
                    <span>{{ t(`copyGroups.${group.id}`) }}</span>
                </label>
            </div>
            <p class="mt-1 text-[10px] leading-snug text-muted-foreground">
                {{ t('copyPanel.selectiveHint') }}
            </p>
        </div>

        <Button
            class="shrink-0 gap-2"
            :disabled="!canCopy"
            @click="emit('executeCopy')"
        >
            <Copy class="size-4" />
            {{ copying ? t('copyPanel.copying') : t('copyPanel.copySettings') }}
        </Button>
    </aside>
</template>
