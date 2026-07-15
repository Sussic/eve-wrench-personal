<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Checkbox } from '@/components/ui/checkbox'
import { TableCell, TableRow } from '@/components/ui/table'
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSub,
    DropdownMenuSubContent,
    DropdownMenuSubTrigger,
    DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
    User,
    Rocket,
    MoreHorizontal,
    ArrowUpFromLine,
    Trash2,
    ArrowDownToLine,
} from 'lucide-vue-next'
import type { BackupEntry, SettingsEntry } from '@/types'
import { getServerColor, getServerShortName } from '@/types'
import { useI18n } from '@/composables/useI18n'

const { t } = useI18n()

defineProps<{
    backup: BackupEntry
    isSource: boolean
    selected: boolean
    targets: SettingsEntry[]
}>()

const emit = defineEmits<{
    setSource: [backup: BackupEntry]
    delete: [backup: BackupEntry]
    apply: [backup: BackupEntry, target: SettingsEntry]
    toggleSelect: [checked: boolean]
}>()
</script>

<template>
    <TableRow
        :class="{
            'bg-primary/20': isSource,
            'bg-muted/50': selected && !isSource,
        }"
    >
        <TableCell class="w-8">
            <Checkbox
                :model-value="selected"
                @update:model-value="emit('toggleSelect', $event === true)"
            />
        </TableCell>
        <TableCell class="w-8">
            <div class="flex size-6 items-center justify-center">
                <Rocket
                    v-if="backup.kind === 'char'"
                    class="size-4 text-muted-foreground"
                />
                <User v-else class="size-4 text-muted-foreground" />
            </div>
        </TableCell>
        <TableCell>
            <div class="flex flex-col">
                <div class="flex items-center gap-1.5">
                    <span>{{ backup.name }}</span>
                    <Badge
                        v-if="backup.name.startsWith('pre-')"
                        variant="secondary"
                        class="px-1 py-0 text-[9px]"
                    >
                        {{ t('backup.automatic') }}
                    </Badge>
                </div>
                <div
                    class="flex items-center gap-1.5 text-xs text-muted-foreground"
                >
                    <span>{{
                        backup.original_name || backup.original_id
                    }}</span>
                    <span>· {{ backup.profile }}</span>
                    <Badge
                        variant="outline"
                        class="px-1 py-0 text-[9px]"
                        :style="{
                            borderColor: getServerColor(backup.server),
                            color: getServerColor(backup.server),
                        }"
                    >
                        {{ getServerShortName(backup.server) }}
                    </Badge>
                </div>
            </div>
        </TableCell>
        <TableCell class="text-muted-foreground">{{
            backup.relative_time
        }}</TableCell>
        <TableCell class="w-12 text-right">
            <DropdownMenu>
                <DropdownMenuTrigger as-child>
                    <Button variant="ghost" size="icon">
                        <MoreHorizontal class="size-4" />
                    </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                    <DropdownMenuItem
                        :disabled="isSource"
                        @select="emit('setSource', backup)"
                    >
                        <ArrowUpFromLine class="mr-2 size-4" />
                        {{ t('actions.useAsSource') }}
                    </DropdownMenuItem>
                    <DropdownMenuSub v-if="targets.length">
                        <DropdownMenuSubTrigger>
                            <ArrowDownToLine class="mr-2 size-4" />
                            {{ t('actions.applyTo') }}
                        </DropdownMenuSubTrigger>
                        <DropdownMenuSubContent
                            class="max-h-64 overflow-y-auto"
                        >
                            <DropdownMenuItem
                                v-for="target in targets"
                                :key="target.path"
                                @select="emit('apply', backup, target)"
                            >
                                {{ target.display_name }}
                            </DropdownMenuItem>
                        </DropdownMenuSubContent>
                    </DropdownMenuSub>
                    <DropdownMenuItem
                        class="text-destructive"
                        @select="emit('delete', backup)"
                    >
                        <Trash2 class="mr-2 size-4" />
                        {{ t('actions.delete') }}
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>
        </TableCell>
    </TableRow>
</template>
