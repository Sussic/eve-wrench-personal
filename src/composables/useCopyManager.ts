import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open, save } from '@tauri-apps/plugin-dialog'
import { load } from '@tauri-apps/plugin-store'
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
} from '@/types'
import { isBackup } from '@/types'
import { useConfirm } from './useConfirm'
import { usePrompt } from './usePrompt'
import { useI18n } from './useI18n'

const appData = ref<AppData | null>(null)
const loading = ref(true)
const copying = ref(false)
const source = ref<SourceItem | null>(null)
const targets = ref<SettingsEntry[]>([])
const customEvePath = ref<string | null>(null)
const importAnalysis = ref<ImportAnalysis | null>(null)
const importFilePath = ref<string | null>(null)
const showImportDialog = ref(false)
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

    const sourceKind = computed<SettingsKind | null>(() => {
        if (!source.value) return null
        return source.value.kind
    })

    const canCopy = computed(() => {
        return (
            source.value !== null && targets.value.length > 0 && !copying.value
        )
    })

    const hasData = computed(() => {
        return (
            appData.value &&
            (appData.value.servers.length > 0 ||
                appData.value.backups.length > 0)
        )
    })

    async function loadData(showToast = false) {
        loading.value = true
        try {
            appData.value = await invoke<AppData>('get_app_data', {
                customEvePath: customEvePath.value,
            })
            if (showToast) {
                const serverCount = appData.value?.servers.length || 0
                const backupCount = appData.value?.backups.length || 0
                if (serverCount > 0 || backupCount > 0) {
                    toast.success(t('toast.dataRefreshed'), {
                        description: t('toast.dataRefreshedDesc', { servers: serverCount, backups: backupCount }),
                    })
                }
            }
        } catch (e: unknown) {
            toast.error(t('toast.loadDataFailed'), { description: String(e) })
        } finally {
            loading.value = false
        }
    }

    async function loadCustomPath() {
        try {
            const store = await load('settings.json')
            customEvePath.value =
                (await store.get<string>('customEvePath')) ?? null
        } catch {
            customEvePath.value = null
        }
    }

    async function selectCustomEvePath() {
        const selected = await open({
            directory: true,
            multiple: false,
            title: t('dialog.selectEveFolder'),
        })
        if (!selected) return

        try {
            const store = await load('settings.json')
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
            const store = await load('settings.json')
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
        await loadCustomPath()
        await loadData()
    }

    function setSource(item: SourceItem) {
        const newKind = item.kind
        if (source.value && sourceKind.value !== newKind) {
            targets.value = []
        }
        source.value = item
        targets.value = targets.value.filter((t) => t.kind === newKind)
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

    async function executeCopy() {
        if (!source.value || targets.value.length === 0) return

        const confirmed = await confirm({
            title: t('dialog.copySettings'),
            description: t('dialog.copySettingsDesc', { source: source.value.display_name, count: targets.value.length }),
            confirmText: t('dialog.copy'),
        })
        if (!confirmed) return

        copying.value = true
        try {
            const sourcePath = source.value.path
            const targetPaths = targets.value.map((t) => t.path)
            const count = await invoke<number>('copy_settings', {
                sourcePath,
                targetPaths,
            })
            toast.success(t('toast.settingsCopied'), {
                description: t('toast.settingsCopiedDesc', { count }),
            })
            targets.value = []
        } catch (e: unknown) {
            toast.error(t('toast.copyFailed'), { description: String(e) })
        } finally {
            copying.value = false
        }
    }

    async function createBackup(entry: SettingsEntry) {
        const name = await prompt({
            title: t('dialog.createBackup'),
            description: t('dialog.createBackupDesc', { name: entry.display_name }),
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
                description: t('toast.backupDeletedDesc', { name: backup.name }),
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

    function getBackupsForEntry(entry: SettingsEntry): BackupEntry[] {
        if (!appData.value) return []
        return appData.value.backups.filter(
            (b) => b.kind === entry.kind && b.original_id === entry.id
        )
    }

    async function restoreBackup(entry: SettingsEntry, backup: BackupEntry) {
        const confirmed = await confirm({
            title: t('dialog.restoreBackup'),
            description: t('dialog.restoreBackupDesc', { backup: backup.name, target: entry.display_name }),
            confirmText: t('dialog.restore'),
            destructive: true,
        })
        if (!confirmed) return

        try {
            await invoke<number>('copy_settings', {
                sourcePath: backup.path,
                targetPaths: [entry.path],
            })
            toast.success(t('toast.backupRestored'), {
                description: t('toast.backupRestoredDesc', { name: backup.name }),
            })
        } catch (e: unknown) {
            toast.error(t('toast.restoreFailed'), { description: String(e) })
        }
    }

    async function applyBackup(backup: BackupEntry, target: SettingsEntry) {
        const confirmed = await confirm({
            title: t('dialog.applyBackup'),
            description: t('dialog.applyBackupDesc', { backup: backup.name, target: target.display_name }),
            confirmText: t('dialog.apply'),
            destructive: true,
        })
        if (!confirmed) return

        try {
            await invoke<number>('copy_settings', {
                sourcePath: backup.path,
                targetPaths: [target.path],
            })
            toast.success(t('toast.backupApplied'), {
                description: t('toast.backupAppliedDesc', { backup: backup.name, target: target.display_name }),
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
                description: t('toast.settingsExportedDesc', { count: result.file_count, path: result.path }),
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
            toast.error(t('toast.importAnalysisFailed'), { description: String(e) })
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
                description: t('toast.settingsImportedDesc', { imported: result.imported_count, skipped: result.skipped_count, backedUp: result.backed_up_count }),
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
                description: t('toast.settingUpdatedDesc', { status: enabled ? t('common.enabled') : t('common.disabled') }),
            })
        } catch (e: unknown) {
            toast.error(t('toast.updateSettingFailed'), { description: String(e) })
        }
    }

    return {
        appData,
        loading,
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
        removeTarget,
        clearTargets,
        addAllFromProfile,
        executeCopy,
        createBackup,
        deleteBackup,
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
    }
}
