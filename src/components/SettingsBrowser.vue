<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useStorage } from '@vueuse/core'
import {
    Archive,
    ChevronUp,
    ChevronDown,
    ListPlus,
    Search,
    ShieldCheck,
    Trash2,
    X,
} from 'lucide-vue-next'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import ServerSection from './ServerSection.vue'
import BackupItem from './BackupItem.vue'
import type {
    AppData,
    ProfileData,
    SettingsEntry,
    SettingsKind,
    BackupEntry,
    ServerId,
} from '@/types'
import { getServerColor } from '@/types'
import { useI18n } from '@/composables/useI18n'
import { backupMatches, settingsEntryMatches } from '@/lib/settingsSearch'

const props = defineProps<{
    appData: AppData
    sourceKind: SettingsKind | null
    isSource: (entry: SettingsEntry) => boolean
    isTarget: (entry: SettingsEntry) => boolean
    isBackupSource: (backup: BackupEntry) => boolean
}>()

const emit = defineEmits<{
    setSource: [entry: SettingsEntry]
    addTarget: [entry: SettingsEntry]
    removeTarget: [entry: SettingsEntry]
    backup: [entry: SettingsEntry]
    restore: [entry: SettingsEntry, backup: BackupEntry]
    applyBackup: [backup: BackupEntry, target: SettingsEntry]
    addAllFromProfile: [profile: ProfileData, kind: SettingsKind]
    setBackupSource: [backup: BackupEntry]
    deleteBackup: [backup: BackupEntry]
    deleteBackups: [backups: BackupEntry[]]
    addVisibleTargets: [entries: SettingsEntry[]]
    activeServerChanged: [serverId: ServerId | null]
    refresh: []
    setBracketsAlwaysShow: [serverPath: string, enabled: boolean]
    openRecovery: []
}>()

const { t } = useI18n()
const activeTab = useStorage<string>(
    'eve-wrench-active-tab',
    props.appData.servers[0]?.info.id || ''
)
const lastActiveServer = useStorage<ServerId | null>(
    'eve-wrench-active-server',
    props.appData.servers[0]?.info.id ?? null
)
const searchQuery = ref('')

function focusSearch(event: KeyboardEvent) {
    if (!(event.ctrlKey || event.metaKey) || event.key.toLowerCase() !== 'f')
        return
    event.preventDefault()
    document.querySelector<HTMLInputElement>('[data-settings-search]')?.focus()
}

onMounted(() => window.addEventListener('keydown', focusSearch))
onUnmounted(() => window.removeEventListener('keydown', focusSearch))

watch(
    [
        () => props.appData.servers,
        () =>
            props.appData.backups.length +
            props.appData.recovery_snapshots.length,
    ],
    ([servers, backupCount]) => {
        if (!servers.length && backupCount > 0) {
            activeTab.value = 'backups'
            return
        }
        const backupTabValid = activeTab.value === 'backups' && backupCount > 0
        if (
            servers.length &&
            !backupTabValid &&
            !servers.find((s) => s.info.id === activeTab.value)
        ) {
            activeTab.value = servers[0].info.id
        }
    },
    { immediate: true }
)

watch(
    activeTab,
    (tab) => {
        const server = props.appData.servers.find(
            (candidate) => candidate.info.id === tab
        )
        if (server) lastActiveServer.value = server.info.id
        emit('activeServerChanged', lastActiveServer.value)
    },
    { immediate: true }
)

const filteredServers = computed(() =>
    props.appData.servers.map((server) => ({
        ...server,
        profiles: server.profiles
            .map((profile) => ({
                ...profile,
                accounts: profile.accounts.filter((entry) =>
                    settingsEntryMatches(entry, profile, searchQuery.value)
                ),
                characters: profile.characters.filter((entry) =>
                    settingsEntryMatches(entry, profile, searchQuery.value)
                ),
            }))
            .filter(
                (profile) =>
                    profile.accounts.length > 0 || profile.characters.length > 0
            ),
    }))
)

const activeFilteredServer = computed(() =>
    filteredServers.value.find((server) => server.info.id === activeTab.value)
)

const visibleTargetEntries = computed(() => {
    if (!props.sourceKind || !activeFilteredServer.value) return []
    return activeFilteredServer.value.profiles.flatMap((profile) =>
        props.sourceKind === 'char' ? profile.characters : profile.accounts
    )
})

type SortColumn = 'name' | 'time'
type SortDirection = 'asc' | 'desc'

const backupSortCol = ref<SortColumn>('time')
const backupSortDir = ref<SortDirection>('desc')

function toggleBackupSort(col: SortColumn) {
    if (backupSortCol.value === col) {
        backupSortDir.value = backupSortDir.value === 'asc' ? 'desc' : 'asc'
    } else {
        backupSortCol.value = col
        backupSortDir.value = col === 'time' ? 'desc' : 'asc'
    }
}

const sortedBackups = computed(() => {
    return props.appData.backups
        .filter((backup) => backupMatches(backup, searchQuery.value))
        .sort((a, b) => {
            let cmp = 0
            if (backupSortCol.value === 'name') {
                cmp = a.name.localeCompare(b.name)
            } else {
                cmp = a.timestamp - b.timestamp
            }
            return backupSortDir.value === 'asc' ? cmp : -cmp
        })
})

const visibleResultCount = computed(() => {
    if (activeTab.value === 'backups') return sortedBackups.value.length
    if (!activeFilteredServer.value) return 0
    return activeFilteredServer.value.profiles.reduce(
        (count, profile) =>
            count + profile.accounts.length + profile.characters.length,
        0
    )
})

// ── Backup multi-select ──────────────────────────────────────────────────
const selectedBackupIds = ref<Set<string>>(new Set())

const selectedBackups = computed(() =>
    props.appData.backups.filter((b) => selectedBackupIds.value.has(b.id))
)
const allBackupsSelected = computed(
    () =>
        sortedBackups.value.length > 0 &&
        sortedBackups.value.every((b) => selectedBackupIds.value.has(b.id))
)

function toggleBackup(id: string, checked: boolean) {
    const next = new Set(selectedBackupIds.value)
    if (checked) next.add(id)
    else next.delete(id)
    selectedBackupIds.value = next
}

function toggleAllBackups(checked: boolean) {
    selectedBackupIds.value = checked
        ? new Set(sortedBackups.value.map((b) => b.id))
        : new Set()
}

function deleteSelectedBackups() {
    emit('deleteBackups', selectedBackups.value)
}

// Drop selections whose backups no longer exist (e.g. after a delete)
watch(
    () => props.appData.backups,
    (backups) => {
        const ids = new Set(backups.map((b) => b.id))
        if ([...selectedBackupIds.value].some((id) => !ids.has(id))) {
            selectedBackupIds.value = new Set(
                [...selectedBackupIds.value].filter((id) => ids.has(id))
            )
        }
    }
)

function getTargetsForBackup(backup: BackupEntry): SettingsEntry[] {
    const entries: SettingsEntry[] = []
    for (const server of props.appData.servers) {
        for (const profile of server.profiles) {
            const items =
                backup.kind === 'char' ? profile.characters : profile.accounts
            entries.push(...items)
        }
    }
    return entries
}

function formatTimestamp(timestamp: number): string {
    return new Intl.DateTimeFormat(undefined, {
        dateStyle: 'medium',
        timeStyle: 'short',
    }).format(new Date(timestamp * 1000))
}
</script>

<template>
    <section class="flex flex-1 flex-col overflow-hidden text-sm">
        <Tabs v-model="activeTab" class="flex flex-1 flex-col overflow-hidden">
            <div class="shrink-0 px-4 pt-4">
                <TabsList
                    class="h-9 w-full justify-start gap-1 bg-transparent p-0"
                >
                    <TabsTrigger
                        v-for="server in appData.servers"
                        :key="server.info.id"
                        :value="server.info.id"
                        class="gap-1.5 data-[state=active]:bg-muted"
                    >
                        <span
                            class="size-2 rounded-full"
                            :style="{
                                backgroundColor: getServerColor(server.info.id),
                            }"
                        />
                        <span>{{ server.info.name }}</span>
                    </TabsTrigger>
                    <TabsTrigger
                        v-if="
                            appData.backups.length ||
                            appData.recovery_snapshots.length
                        "
                        value="backups"
                        class="gap-1.5 data-[state=active]:bg-muted"
                    >
                        <Archive class="size-3" />
                        <span>{{ t('titleBar.backups') }}</span>
                        <span class="text-xs text-muted-foreground"
                            >({{
                                appData.backups.length +
                                appData.recovery_snapshots.length
                            }})</span
                        >
                    </TabsTrigger>
                </TabsList>
                <div class="mt-3 flex items-center gap-2">
                    <div class="relative min-w-0 flex-1">
                        <Search
                            class="pointer-events-none absolute left-2.5 top-1/2 size-4 -translate-y-1/2 text-muted-foreground"
                        />
                        <Input
                            v-model="searchQuery"
                            data-settings-search
                            class="h-8 pl-8 pr-8"
                            :placeholder="t('search.placeholder')"
                            @keyup.escape="searchQuery = ''"
                        />
                        <Button
                            v-if="searchQuery"
                            variant="ghost"
                            size="icon"
                            class="absolute right-0 top-0 size-8"
                            :title="t('search.clear')"
                            @click="searchQuery = ''"
                        >
                            <X class="size-3.5" />
                        </Button>
                    </div>
                    <span
                        v-if="searchQuery"
                        class="shrink-0 text-xs text-muted-foreground"
                    >
                        {{ t('search.results', { count: visibleResultCount }) }}
                    </span>
                    <Button
                        v-if="
                            sourceKind &&
                            activeTab !== 'backups' &&
                            visibleTargetEntries.length
                        "
                        variant="outline"
                        size="sm"
                        class="h-8 shrink-0"
                        @click="emit('addVisibleTargets', visibleTargetEntries)"
                    >
                        <ListPlus class="mr-1.5 size-3.5" />
                        {{
                            t('search.addVisible', {
                                count: visibleTargetEntries.length,
                            })
                        }}
                    </Button>
                </div>
            </div>

            <div class="flex-1 overflow-y-auto p-4">
                <TabsContent
                    v-for="server in filteredServers"
                    :key="server.info.id"
                    :value="server.info.id"
                    class="mt-0"
                >
                    <ServerSection
                        v-if="!searchQuery || server.profiles.length > 0"
                        :server="server"
                        :source-kind="sourceKind"
                        :is-source="isSource"
                        :is-target="isTarget"
                        :all-backups="appData.backups"
                        @set-source="emit('setSource', $event)"
                        @add-target="emit('addTarget', $event)"
                        @remove-target="emit('removeTarget', $event)"
                        @backup="emit('backup', $event)"
                        @restore="
                            (entry, backup) => emit('restore', entry, backup)
                        "
                        @add-all-from-profile="
                            (p, k) => emit('addAllFromProfile', p, k)
                        "
                        @refresh="emit('refresh')"
                        @set-brackets-always-show="
                            (path, enabled) =>
                                emit('setBracketsAlwaysShow', path, enabled)
                        "
                    />
                    <div
                        v-if="searchQuery && server.profiles.length === 0"
                        class="py-16 text-center text-sm text-muted-foreground"
                    >
                        {{ t('search.noResults') }}
                    </div>
                </TabsContent>

                <TabsContent value="backups" class="mt-0">
                    <section
                        v-if="appData.recovery_snapshots.length"
                        class="mb-5"
                    >
                        <div class="mb-2 flex items-end gap-2">
                            <div>
                                <div class="text-lg font-semibold">
                                    {{ t('backup.fullTitle') }}
                                </div>
                                <div class="text-xs text-muted-foreground">
                                    {{ t('backup.fullDescription') }}
                                </div>
                            </div>
                            <Button
                                variant="outline"
                                size="sm"
                                class="ml-auto h-7"
                                @click="emit('openRecovery')"
                            >
                                {{ t('recovery.restoreAll') }}
                            </Button>
                        </div>
                        <div class="grid gap-2 sm:grid-cols-2 xl:grid-cols-3">
                            <button
                                v-for="snapshot in appData.recovery_snapshots.slice(
                                    0,
                                    3
                                )"
                                :key="snapshot.id"
                                type="button"
                                class="rounded-md border p-2 text-left hover:bg-muted/40"
                                @click="emit('openRecovery')"
                            >
                                <div
                                    class="flex items-center gap-1.5 text-xs font-medium"
                                >
                                    <ShieldCheck
                                        class="size-3.5 text-primary"
                                    />
                                    {{ formatTimestamp(snapshot.timestamp) }}
                                </div>
                                <div
                                    class="mt-0.5 text-[10px] text-muted-foreground"
                                >
                                    {{
                                        t('recovery.snapshotCounts', {
                                            accounts: snapshot.account_count,
                                            characters:
                                                snapshot.character_count,
                                            profiles: snapshot.profile_count,
                                        })
                                    }}
                                </div>
                            </button>
                        </div>
                    </section>
                    <div class="mb-3 flex items-center gap-2">
                        <div>
                            <div class="text-lg font-semibold">
                                {{ t('backup.individualTitle') }}
                            </div>
                            <div class="text-xs text-muted-foreground">
                                {{ t('backup.individualDescription') }}
                            </div>
                        </div>
                        <span
                            class="rounded-full bg-muted px-2 py-0.5 text-xs text-muted-foreground"
                        >
                            {{ appData.backups.length }}
                        </span>
                        <div
                            v-if="selectedBackups.length"
                            class="ml-auto flex items-center gap-2"
                        >
                            <span class="text-sm text-muted-foreground">
                                {{
                                    t('backup.selectedCount', {
                                        count: selectedBackups.length,
                                    })
                                }}
                            </span>
                            <Button
                                variant="destructive"
                                size="sm"
                                class="h-7"
                                @click="deleteSelectedBackups"
                            >
                                <Trash2 class="mr-1 size-3.5" />
                                {{ t('backup.deleteSelected') }}
                            </Button>
                        </div>
                    </div>
                    <div class="rounded-md border">
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead class="w-8">
                                        <Checkbox
                                            :model-value="allBackupsSelected"
                                            @update:model-value="
                                                toggleAllBackups(
                                                    $event === true
                                                )
                                            "
                                        />
                                    </TableHead>
                                    <TableHead class="w-8"></TableHead>
                                    <TableHead
                                        class="cursor-pointer select-none"
                                        @click="toggleBackupSort('time')"
                                    >
                                        <div class="flex items-center gap-1">
                                            {{ t('backup.created') }}
                                            <ChevronUp
                                                v-if="
                                                    backupSortCol === 'time' &&
                                                    backupSortDir === 'asc'
                                                "
                                                class="size-3"
                                            />
                                            <ChevronDown
                                                v-else-if="
                                                    backupSortCol === 'time' &&
                                                    backupSortDir === 'desc'
                                                "
                                                class="size-3"
                                            />
                                        </div>
                                    </TableHead>
                                    <TableHead>{{
                                        t('backup.identity')
                                    }}</TableHead>
                                    <TableHead
                                        class="cursor-pointer select-none"
                                        @click="toggleBackupSort('name')"
                                    >
                                        <div class="flex items-center gap-1">
                                            {{ t('backup.reason') }}
                                            <ChevronUp
                                                v-if="
                                                    backupSortCol === 'name' &&
                                                    backupSortDir === 'asc'
                                                "
                                                class="size-3"
                                            />
                                            <ChevronDown
                                                v-else-if="
                                                    backupSortCol === 'name' &&
                                                    backupSortDir === 'desc'
                                                "
                                                class="size-3"
                                            />
                                        </div>
                                    </TableHead>
                                    <TableHead class="w-20"></TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                <BackupItem
                                    v-for="backup in sortedBackups"
                                    :key="backup.id"
                                    :backup="backup"
                                    :is-source="isBackupSource(backup)"
                                    :selected="selectedBackupIds.has(backup.id)"
                                    :targets="getTargetsForBackup(backup)"
                                    @set-source="
                                        emit('setBackupSource', $event)
                                    "
                                    @delete="emit('deleteBackup', $event)"
                                    @apply="(b, t) => emit('applyBackup', b, t)"
                                    @toggle-select="
                                        toggleBackup(backup.id, $event)
                                    "
                                />
                                <TableRow v-if="sortedBackups.length === 0">
                                    <TableCell
                                        colspan="6"
                                        class="h-24 text-center font-normal text-muted-foreground"
                                    >
                                        {{
                                            searchQuery
                                                ? t('search.noResults')
                                                : t('backup.noIndividual')
                                        }}
                                    </TableCell>
                                </TableRow>
                            </TableBody>
                        </Table>
                    </div>
                </TabsContent>
            </div>
        </Tabs>
    </section>
</template>
