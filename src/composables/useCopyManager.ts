import { ref, computed } from 'vue'
import { useStorage } from '@vueuse/core'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open, save } from '@tauri-apps/plugin-dialog'
import { toast } from 'vue-sonner'
import type {
    SourceItem,
    SettingsEntry,
    SettingsKind,
    ProfileData,
    BackupEntry,
    AppData,
    ExportResult,
    ImportAnalysis,
    ImportResultInfo,
    BatchMutationResult,
    BulkTargetScope,
    ServerId,
} from '@/types'
import { getServerShortName, isBackup } from '@/types'
import { defaultGroupSelection, groupsForKind } from '@/lib/copyGroups'
import { collectEntriesByKind, mergeUniqueTargets } from '@/lib/targetSelection'
import {
    settingsStore,
    getAutoBackup,
    setAutoBackup as persistAutoBackup,
} from '@/lib/settingsStore'
import { useConfirm } from './useConfirm'
import { usePrompt } from './usePrompt'
import { useI18n } from './useI18n'
import { useEveRunning } from './useEveRunning'

const appData = ref<AppData | null>(null)
const loading = ref(true)
const loadError = ref<string | null>(null)
const copying = ref(false)
const source = ref<SourceItem | null>(null)
const targets = ref<SettingsEntry[]>([])
const customEvePath = ref<string | null>(null)
const importAnalysis = ref<ImportAnalysis | null>(null)
const importFilePath = ref<string | null>(null)
const showImportDialog = ref(false)
const copyGroupSelection = useStorage<Record<string, boolean>>(
    'eve-wrench-copy-groups',
    defaultGroupSelection(),
    undefined,
    { mergeDefaults: true }
)
const autoBackup = ref(true)
let listenerSetup = false

async function setupListener(loadDataFn: () => Promise<void>) {
    if (listenerSetup) return
    listenerSetup = true
    await listen('data-changed', () => {
        loadDataFn()
    })
}

export function useCopyManager() {
    const { confirm } = useConfirm()
    const { prompt } = usePrompt()
    const { t } = useI18n()
    const { eveRunning } = useEveRunning()

    const sourceKind = computed<SettingsKind | null>(() => {
        if (!source.value) return null
        return source.value.kind
    })

    // Unchecked groups are excluded: the target keeps its own version of
    // them, while everything else in the file copies over.
    const excludedCopyGroups = computed<string[]>(() => {
        if (!sourceKind.value) return []
        return groupsForKind(sourceKind.value)
            .filter((g) => !copyGroupSelection.value[g.id])
            .map((g) => g.id)
    })

    const hasSelectedCopyGroup = computed(() => {
        if (!sourceKind.value) return false
        return groupsForKind(sourceKind.value).some(
            (group) => copyGroupSelection.value[group.id]
        )
    })

    const canCopy = computed(() => {
        return (
            source.value !== null &&
            targets.value.length > 0 &&
            hasSelectedCopyGroup.value &&
            !copying.value &&
            !eveRunning.value
        )
    })

    const hasData = computed(() => {
        return (
            appData.value &&
            (appData.value.servers.length > 0 ||
                appData.value.backups.length > 0)
        )
    })

    function reconcileSelections(data: AppData) {
        const entriesByPath = new Map<string, SettingsEntry>()
        for (const server of data.servers) {
            for (const profile of server.profiles) {
                for (const entry of [
                    ...profile.accounts,
                    ...profile.characters,
                ]) {
                    entriesByPath.set(entry.path, entry)
                }
            }
        }
        const backupsByPath = new Map(
            data.backups.map((backup) => [backup.path, backup])
        )
        let sourceRemoved = false
        if (source.value) {
            const rebound = isBackup(source.value)
                ? backupsByPath.get(source.value.path)
                : entriesByPath.get(source.value.path)
            if (rebound) source.value = rebound
            else {
                source.value = null
                sourceRemoved = true
            }
        }

        const previousTargetCount = targets.value.length
        targets.value = targets.value
            .map((target) => entriesByPath.get(target.path))
            .filter((target): target is SettingsEntry => !!target)
        if (sourceRemoved) targets.value = []

        const removedTargets = previousTargetCount - targets.value.length
        if (sourceRemoved) {
            toast.warning(t('toast.selectionSourceRemoved'), {
                description: t('toast.selectionSourceRemovedDesc'),
            })
        } else if (removedTargets > 0) {
            toast.warning(t('toast.selectionTargetsRemoved'), {
                description: t('toast.selectionTargetsRemovedDesc', {
                    count: removedTargets,
                }),
            })
        }
    }

    async function loadData(showToast = false) {
        loading.value = true
        loadError.value = null
        try {
            const data = await invoke<AppData>('get_app_data', {
                customEvePath: customEvePath.value,
            })
            reconcileSelections(data)
            appData.value = data
            if (showToast) {
                const serverCount = appData.value?.servers.length || 0
                const backupCount = appData.value?.backups.length || 0
                if (serverCount > 0 || backupCount > 0) {
                    toast.success(t('toast.dataRefreshed'), {
                        description: t('toast.dataRefreshedDesc', {
                            servers: serverCount,
                            backups: backupCount,
                        }),
                    })
                }
            }
        } catch (e: unknown) {
            loadError.value = String(e)
            toast.error(t('toast.loadDataFailed'), {
                description: loadError.value,
            })
        } finally {
            loading.value = false
        }
    }

    async function loadSettings() {
        try {
            const store = await settingsStore()
            customEvePath.value =
                (await store.get<string>('customEvePath')) ?? null
        } catch {
            customEvePath.value = null
        }
        autoBackup.value = await getAutoBackup()
    }

    async function setAutoBackup(enabled: boolean) {
        autoBackup.value = enabled
        await persistAutoBackup(enabled)
    }

    async function selectCustomEvePath() {
        const selected = await open({
            directory: true,
            multiple: false,
            title: t('dialog.selectEveFolder'),
        })
        if (!selected) return

        try {
            const store = await settingsStore()
            await store.set('customEvePath', selected)
            customEvePath.value = selected
            toast.success(t('toast.customPathSet'), {
                description: selected,
            })
            await loadData()
        } catch (e: unknown) {
            toast.error(t('toast.setPathFailed'), { description: String(e) })
        }
    }

    async function clearCustomEvePath() {
        try {
            const store = await settingsStore()
            await store.delete('customEvePath')
            customEvePath.value = null
            toast.success(t('toast.pathReset'), {
                description: t('toast.pathResetDesc'),
            })
            await loadData()
        } catch (e: unknown) {
            toast.error(t('toast.resetPathFailed'), { description: String(e) })
        }
    }

    async function init() {
        await setupListener(loadData)
        await loadSettings()
        await loadData()
    }

    function setSource(item: SourceItem) {
        const newKind = item.kind
        if (source.value && sourceKind.value !== newKind) {
            targets.value = []
        }
        source.value = item
        const sourcePath = isBackup(item) ? null : item.path
        targets.value = targets.value.filter(
            (target) => target.kind === newKind && target.path !== sourcePath
        )
    }

    function clearSource() {
        source.value = null
    }

    function addTarget(entry: SettingsEntry) {
        if (!source.value) {
            toast.error(t('toast.noSourceSelected'), {
                description: t('toast.noSourceSelectedDesc'),
            })
            return
        }
        if (entry.kind !== sourceKind.value) {
            toast.error(t('toast.typeMismatch'), {
                description: t('toast.typeMismatchDesc'),
            })
            return
        }
        if (!isBackup(source.value) && source.value.path === entry.path) {
            toast.error(t('toast.invalidTarget'), {
                description: t('toast.invalidTargetDesc'),
            })
            return
        }
        if (targets.value.some((t) => t.path === entry.path)) {
            return
        }
        targets.value.push(entry)
    }

    function addTargets(entries: SettingsEntry[]) {
        if (!source.value) {
            toast.error(t('toast.noSourceSelected'), {
                description: t('toast.noSourceSelectedDesc'),
            })
            return
        }
        const excludedPath = isBackup(source.value) ? null : source.value.path
        targets.value = mergeUniqueTargets(
            targets.value,
            entries.filter((entry) => entry.kind === sourceKind.value),
            excludedPath
        )
    }

    function removeTarget(entry: SettingsEntry) {
        targets.value = targets.value.filter((t) => t.path !== entry.path)
    }

    function clearTargets() {
        targets.value = []
    }

    function addAllFromProfile(profile: ProfileData, kind: SettingsKind) {
        if (!source.value) {
            toast.error(t('toast.noSourceSelected'), {
                description: t('toast.noSourceSelectedDesc'),
            })
            return
        }
        if (sourceKind.value !== kind) return

        const items = kind === 'char' ? profile.characters : profile.accounts
        for (const item of items) {
            if (!isBackup(source.value) && source.value.path === item.path)
                continue
            if (targets.value.some((t) => t.path === item.path)) continue
            targets.value.push(item)
        }
    }

    function addAllTargets(
        scope: BulkTargetScope,
        activeServerId: ServerId | null
    ) {
        if (!source.value || !appData.value) {
            toast.error(t('toast.noSourceSelected'), {
                description: t('toast.noSourceSelectedDesc'),
            })
            return
        }

        if (scope === 'server' && !activeServerId) return
        const candidates = collectEntriesByKind(
            appData.value,
            source.value.kind,
            scope === 'server' ? activeServerId : null
        )
        const excludedPath = isBackup(source.value) ? null : source.value.path
        targets.value = mergeUniqueTargets(
            targets.value,
            candidates,
            excludedPath
        )
    }

    async function executeCopy() {
        if (!source.value || targets.value.length === 0) return

        const targetBreakdown = new Map<string, number>()
        for (const target of targets.value) {
            const label = getServerShortName(target.server)
            targetBreakdown.set(label, (targetBreakdown.get(label) ?? 0) + 1)
        }
        const includedGroups = groupsForKind(source.value.kind)
            .filter((group) => copyGroupSelection.value[group.id])
            .map((group) => t(`copyGroups.${group.id}`))
            .join(', ')
        const targetSummary = [...targetBreakdown]
            .map(([server, count]) => `${server}: ${count}`)
            .join(' · ')
        const sourceServer = isBackup(source.value) ? null : source.value.server
        const crossesServers =
            sourceServer !== null &&
            targets.value.some((target) => target.server !== sourceServer)
        const description = [
            t('dialog.copySettingsDesc', {
                source: source.value.display_name,
                count: targets.value.length,
            }),
            t('dialog.copyTargetsSummary', { targets: targetSummary }),
            t('dialog.copyGroupsSummary', { groups: includedGroups }),
            t('dialog.copyBackupSummary', {
                status: autoBackup.value
                    ? t('common.enabled')
                    : t('common.disabled'),
            }),
            ...(crossesServers ? [t('dialog.crossServerWarning')] : []),
        ].join('\n')
        const confirmed = await confirm({
            title: t('dialog.copySettings'),
            description,
            confirmText: t('dialog.copy'),
        })
        if (!confirmed) return

        copying.value = true
        try {
            const sourcePath = source.value.path
            const targetPaths = targets.value.map((t) => t.path)
            const result = await invoke<BatchMutationResult>(
                'copy_settings_selective',
                {
                    sourcePath,
                    targetPaths,
                    excludedGroups: excludedCopyGroups.value,
                    backup: autoBackup.value,
                }
            )
            if (result.failed.length) {
                const description = t('toast.settingsCopyPartialDesc', {
                    succeeded: result.succeeded.length,
                    failed: result.failed.length,
                    reason: result.failed[0].error,
                })
                if (result.succeeded.length) {
                    toast.warning(t('toast.settingsCopyPartial'), {
                        description,
                    })
                } else {
                    toast.error(t('toast.copyFailed'), { description })
                }
                const failedPaths = new Set(result.failed.map((f) => f.path))
                targets.value = targets.value.filter((target) =>
                    failedPaths.has(target.path)
                )
            } else {
                toast.success(t('toast.settingsCopied'), {
                    description: t('toast.settingsCopiedDesc', {
                        count: result.succeeded.length,
                    }),
                })
                targets.value = []
            }
        } catch (e: unknown) {
            toast.error(t('toast.copyFailed'), { description: String(e) })
        } finally {
            copying.value = false
        }
    }

    async function createBackup(entry: SettingsEntry) {
        const name = await prompt({
            title: t('dialog.createBackup'),
            description: t('dialog.createBackupDesc', {
                name: entry.display_name,
            }),
            placeholder: t('dialog.backupName'),
            defaultValue: entry.display_name,
            confirmText: t('dialog.create'),
        })
        if (!name) return

        try {
            await invoke('create_backup', {
                sourcePath: entry.path,
                backupName: name,
            })
            toast.success(t('toast.backupCreated'), {
                description: t('toast.backupCreatedDesc', { name }),
            })
        } catch (e: unknown) {
            toast.error(t('toast.backupFailed'), { description: String(e) })
        }
    }

    async function deleteBackup(backup: BackupEntry) {
        const confirmed = await confirm({
            title: t('dialog.deleteBackup'),
            description: t('dialog.deleteBackupDesc', { name: backup.name }),
            confirmText: t('dialog.delete'),
            destructive: true,
        })
        if (!confirmed) return

        try {
            await invoke('delete_backup', { backupPath: backup.path })
            toast.success(t('toast.backupDeleted'), {
                description: t('toast.backupDeletedDesc', {
                    name: backup.name,
                }),
            })
            if (
                source.value &&
                isBackup(source.value) &&
                source.value.id === backup.id
            ) {
                source.value = null
            }
        } catch (e: unknown) {
            toast.error(t('toast.deleteFailed'), { description: String(e) })
        }
    }

    async function deleteBackups(backups: BackupEntry[]) {
        if (backups.length === 0) return
        const confirmed = await confirm({
            title: t('dialog.deleteBackups'),
            description: t('dialog.deleteBackupsDesc', {
                count: backups.length,
            }),
            confirmText: t('dialog.delete'),
            destructive: true,
        })
        if (!confirmed) return

        try {
            const ids = new Set(backups.map((b) => b.id))
            const count = await invoke<number>('delete_backups', {
                backupPaths: backups.map((b) => b.path),
            })
            toast.success(t('toast.backupsDeleted'), {
                description: t('toast.backupsDeletedDesc', { count }),
            })
            if (
                source.value &&
                isBackup(source.value) &&
                ids.has(source.value.id)
            ) {
                source.value = null
            }
        } catch (e: unknown) {
            toast.error(t('toast.deleteFailed'), { description: String(e) })
        }
    }

    function getBackupsForEntry(entry: SettingsEntry): BackupEntry[] {
        if (!appData.value) return []
        return appData.value.backups.filter(
            (b) => b.kind === entry.kind && b.original_id === entry.id
        )
    }

    async function restoreBackup(entry: SettingsEntry, backup: BackupEntry) {
        const confirmed = await confirm({
            title: t('dialog.restoreBackup'),
            description: t('dialog.restoreBackupDesc', {
                backup: backup.name,
                target: entry.display_name,
            }),
            confirmText: t('dialog.restore'),
            destructive: true,
        })
        if (!confirmed) return

        try {
            const result = await invoke<BatchMutationResult>('copy_settings', {
                sourcePath: backup.path,
                targetPaths: [entry.path],
                backup: autoBackup.value,
            })
            if (result.succeeded.length !== 1) {
                throw new Error(
                    result.failed[0]?.error ??
                        'The settings file was not changed'
                )
            }
            toast.success(t('toast.backupRestored'), {
                description: t('toast.backupRestoredDesc', {
                    name: backup.name,
                }),
            })
        } catch (e: unknown) {
            toast.error(t('toast.restoreFailed'), { description: String(e) })
        }
    }

    async function applyBackup(backup: BackupEntry, target: SettingsEntry) {
        const confirmed = await confirm({
            title: t('dialog.applyBackup'),
            description: t('dialog.applyBackupDesc', {
                backup: backup.name,
                target: target.display_name,
            }),
            confirmText: t('dialog.apply'),
            destructive: true,
        })
        if (!confirmed) return

        try {
            const result = await invoke<BatchMutationResult>('copy_settings', {
                sourcePath: backup.path,
                targetPaths: [target.path],
                backup: autoBackup.value,
            })
            if (result.succeeded.length !== 1) {
                throw new Error(
                    result.failed[0]?.error ??
                        'The settings file was not changed'
                )
            }
            toast.success(t('toast.backupApplied'), {
                description: t('toast.backupAppliedDesc', {
                    backup: backup.name,
                    target: target.display_name,
                }),
            })
        } catch (e: unknown) {
            toast.error(t('toast.applyFailed'), { description: String(e) })
        }
    }

    function isSource(item: SourceItem): boolean {
        if (!source.value) return false
        if (isBackup(item) && isBackup(source.value)) {
            return source.value.id === item.id
        }
        if (!isBackup(item) && !isBackup(source.value)) {
            return source.value.path === item.path
        }
        return false
    }

    function isTarget(entry: SettingsEntry): boolean {
        return targets.value.some((t) => t.path === entry.path)
    }

    function refresh() {
        loadData(true)
    }

    async function exportSettings() {
        const confirmed = await confirm({
            title: t('dialog.exportPrivacyTitle'),
            description: t('dialog.exportPrivacyDesc'),
            confirmText: t('importExport.export'),
        })
        if (!confirmed) return

        const exportPath = await save({
            title: t('dialog.exportSettings'),
            defaultPath: `eve-wrench-export-${Date.now()}.zip`,
            filters: [{ name: 'ZIP Archive', extensions: ['zip'] }],
        })
        if (!exportPath) return

        try {
            const result = await invoke<ExportResult>('export_settings', {
                customEvePath: customEvePath.value,
                exportPath,
            })
            toast.success(t('toast.settingsExported'), {
                description: t('toast.settingsExportedDesc', {
                    count: result.file_count,
                    path: result.path,
                }),
            })
        } catch (e: unknown) {
            toast.error(t('toast.exportFailed'), { description: String(e) })
        }
    }

    async function importSettings() {
        const selected = await open({
            title: t('dialog.importSettings'),
            multiple: false,
            filters: [{ name: 'ZIP Archive', extensions: ['zip'] }],
        })
        if (!selected) return

        try {
            const analysis = await invoke<ImportAnalysis>('analyze_import', {
                importPath: selected,
                customEvePath: customEvePath.value,
            })
            importAnalysis.value = analysis
            importFilePath.value = selected
            showImportDialog.value = true
        } catch (e: unknown) {
            toast.error(t('toast.importAnalysisFailed'), {
                description: String(e),
            })
        }
    }

    async function executeImport(overwritePaths: string[]) {
        if (!importFilePath.value) return

        showImportDialog.value = false

        try {
            const result = await invoke<ImportResultInfo>('execute_import', {
                importPath: importFilePath.value,
                customEvePath: customEvePath.value,
                overwritePaths,
            })
            toast.success(t('toast.settingsImported'), {
                description: t('toast.settingsImportedDesc', {
                    imported: result.imported_count,
                    skipped: result.skipped_count,
                    backedUp: result.backed_up_count,
                }),
            })
        } catch (e: unknown) {
            toast.error(t('toast.importFailed'), { description: String(e) })
        } finally {
            importAnalysis.value = null
            importFilePath.value = null
        }
    }

    function cancelImport() {
        showImportDialog.value = false
        importAnalysis.value = null
        importFilePath.value = null
    }

    async function setBracketsAlwaysShow(serverPath: string, enabled: boolean) {
        try {
            await invoke('set_brackets_always_show', { serverPath, enabled })
            toast.success(t('toast.settingUpdated'), {
                description: t('toast.settingUpdatedDesc', {
                    status: enabled
                        ? t('common.enabled')
                        : t('common.disabled'),
                }),
            })
        } catch (e: unknown) {
            toast.error(t('toast.updateSettingFailed'), {
                description: String(e),
            })
        }
    }

    return {
        appData,
        loading,
        loadError,
        copying,
        source,
        targets,
        sourceKind,
        canCopy,
        hasData,
        customEvePath,
        init,
        refresh,
        setSource,
        clearSource,
        addTarget,
        addTargets,
        removeTarget,
        clearTargets,
        addAllFromProfile,
        addAllTargets,
        executeCopy,
        createBackup,
        deleteBackup,
        deleteBackups,
        autoBackup,
        setAutoBackup,
        getBackupsForEntry,
        restoreBackup,
        applyBackup,
        isSource,
        isTarget,
        setBracketsAlwaysShow,
        selectCustomEvePath,
        clearCustomEvePath,
        exportSettings,
        importSettings,
        executeImport,
        cancelImport,
        importAnalysis,
        showImportDialog,
        copyGroupSelection,
        eveRunning,
    }
}
