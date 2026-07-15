<script setup lang="ts">
import 'vue-sonner/style.css'
import { computed, onMounted, ref } from 'vue'
import { useColorMode } from '@vueuse/core'
import { Toaster } from '@/components/ui/sonner'
import { TooltipProvider } from '@/components/ui/tooltip'
import { AlertTriangle, Rocket, FolderOpen } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import TitleBar from '@/components/TitleBar.vue'
import SettingsBrowser from '@/components/SettingsBrowser.vue'
import CopyPanel from '@/components/CopyPanel.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import PromptDialog from '@/components/PromptDialog.vue'
import ImportDialog from '@/components/ImportDialog.vue'
import RecoveryBar from '@/components/RecoveryBar.vue'
import RecoveryDialog from '@/components/RecoveryDialog.vue'
import FormationVariationDialog from '@/components/FormationVariationDialog.vue'
import { useCopyManager } from '@/composables/useCopyManager'
import { useI18n } from '@/composables/useI18n'
import { isBackup } from '@/types'
import type {
    FormationVariationAxes,
    RecoverySnapshot,
    ServerId,
    SettingsEntry,
} from '@/types'

const colorMode = useColorMode()
const { t } = useI18n()
const {
    appData,
    loading,
    loadError,
    copying,
    source,
    targets,
    sourceKind,
    canCopy,
    canCreateFormationVariants,
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
    recoveryBusy,
    createFullSnapshot,
    restoreFullSnapshot,
    executeFormationVariants,
} = useCopyManager()

const activeServerId = ref<ServerId | null>(null)
const showRecoveryDialog = ref(false)
const showFormationVariationDialog = ref(false)
const formationVariationSource = computed<SettingsEntry | null>(() => {
    if (
        !source.value ||
        isBackup(source.value) ||
        source.value.kind !== 'user'
    ) {
        return null
    }
    return source.value
})

async function applyFormationVariants(options: {
    variedFormationIds: number[]
    variabilityKm: number
    axes: FormationVariationAxes
}) {
    showFormationVariationDialog.value = false
    await executeFormationVariants(options)
}

async function restoreSnapshot(snapshot: RecoverySnapshot) {
    showRecoveryDialog.value = false
    await restoreFullSnapshot(snapshot)
}

function isBackupSource(backup: { id: string }): boolean {
    return !!(
        source.value &&
        isBackup(source.value) &&
        source.value.id === backup.id
    )
}

function toggleDarkMode() {
    colorMode.value = colorMode.value === 'dark' ? 'light' : 'dark'
}

onMounted(init)
</script>

<template>
    <TooltipProvider>
        <div
            class="fixed inset-0 flex flex-col overflow-hidden bg-background"
            :class="colorMode"
        >
            <Toaster
                position="top-center"
                rich-colors
                :theme="colorMode === 'dark' ? 'dark' : 'light'"
            />
            <ConfirmDialog />
            <PromptDialog />
            <ImportDialog
                v-if="importAnalysis"
                :open="showImportDialog"
                :analysis="importAnalysis"
                @confirm="executeImport"
                @cancel="cancelImport"
            />
            <RecoveryDialog
                :open="showRecoveryDialog"
                :snapshots="appData?.recovery_snapshots ?? []"
                :busy="recoveryBusy"
                @cancel="showRecoveryDialog = false"
                @restore="restoreSnapshot"
            />
            <FormationVariationDialog
                :open="showFormationVariationDialog"
                :source="formationVariationSource"
                :targets="targets"
                :busy="copying"
                @cancel="showFormationVariationDialog = false"
                @apply="applyFormationVariants"
            />
            <TitleBar
                :loading="loading"
                :color-mode="colorMode"
                :custom-eve-path="customEvePath"
                :auto-backup="autoBackup"
                @refresh="refresh"
                @toggle-theme="toggleDarkMode"
                @select-eve-path="selectCustomEvePath"
                @clear-eve-path="clearCustomEvePath"
                @export-settings="exportSettings"
                @import-settings="importSettings"
                @set-auto-backup="setAutoBackup"
            />

            <div
                v-if="eveRunning"
                class="flex items-center justify-center gap-2 border-b border-amber-500/30 bg-amber-500/10 px-4 py-2 text-xs font-medium text-amber-700 dark:text-amber-300"
            >
                <AlertTriangle class="size-4" />
                {{ t('safety.eveRunning') }}
            </div>

            <RecoveryBar
                v-if="appData"
                :snapshots="appData.recovery_snapshots"
                :busy="recoveryBusy"
                :eve-running="eveRunning"
                @backup-all="createFullSnapshot"
                @restore-all="showRecoveryDialog = true"
            />

            <main class="flex flex-1 overflow-hidden">
                <div
                    v-if="loading && !appData"
                    class="flex flex-1 items-center justify-center"
                >
                    <div class="flex flex-col items-center gap-3">
                        <div
                            class="size-8 animate-spin rounded-full border-2 border-muted border-t-primary"
                        />
                        <p class="text-sm text-muted-foreground">
                            {{ t('common.loading') }}
                        </p>
                    </div>
                </div>

                <template v-else-if="loadError && !appData">
                    <div class="flex flex-1 items-center justify-center">
                        <div
                            class="flex max-w-md flex-col items-center gap-3 text-center"
                        >
                            <AlertTriangle class="size-12 text-destructive" />
                            <h3 class="font-semibold">
                                {{ t('empty.loadFailed') }}
                            </h3>
                            <p class="text-sm text-muted-foreground">
                                {{ loadError }}
                            </p>
                            <div class="mt-2 flex gap-2">
                                <Button size="sm" @click="refresh">
                                    {{ t('common.retry') }}
                                </Button>
                                <Button
                                    variant="outline"
                                    size="sm"
                                    @click="selectCustomEvePath"
                                >
                                    <FolderOpen class="mr-2 size-4" />
                                    {{ t('settings.changeFolder') }}
                                </Button>
                            </div>
                        </div>
                    </div>
                </template>

                <template v-else-if="!hasData">
                    <div class="flex flex-1 items-center justify-center">
                        <div
                            class="flex flex-col items-center gap-3 text-center"
                        >
                            <Rocket class="size-12 text-muted-foreground" />
                            <h3 class="font-semibold">
                                {{ t('empty.noEveInstallations') }}
                            </h3>
                            <p class="text-sm text-muted-foreground">
                                {{ t('empty.noEveInstallationsDesc') }}
                            </p>
                            <Button
                                variant="outline"
                                size="sm"
                                class="mt-2"
                                @click="selectCustomEvePath"
                            >
                                <FolderOpen class="mr-2 size-4" />
                                {{ t('settings.setCustomPath') }}
                            </Button>
                        </div>
                    </div>
                </template>

                <template v-else-if="appData">
                    <SettingsBrowser
                        :app-data="appData"
                        :source-kind="sourceKind"
                        :is-source="isSource"
                        :is-target="isTarget"
                        :is-backup-source="isBackupSource"
                        @set-source="setSource"
                        @add-target="addTarget"
                        @remove-target="removeTarget"
                        @add-visible-targets="addTargets"
                        @active-server-changed="activeServerId = $event"
                        @backup="createBackup"
                        @restore="restoreBackup"
                        @apply-backup="applyBackup"
                        @add-all-from-profile="addAllFromProfile"
                        @set-backup-source="setSource"
                        @delete-backup="deleteBackup"
                        @delete-backups="deleteBackups"
                        @refresh="refresh"
                        @set-brackets-always-show="setBracketsAlwaysShow"
                        @open-recovery="showRecoveryDialog = true"
                    />

                    <CopyPanel
                        :source="source"
                        :targets="targets"
                        :can-copy="canCopy"
                        :can-create-formation-variants="
                            canCreateFormationVariants
                        "
                        :copying="copying"
                        :group-selection="copyGroupSelection"
                        :active-server-id="activeServerId"
                        @clear-source="clearSource"
                        @remove-target="removeTarget"
                        @clear-targets="clearTargets"
                        @add-all-targets="
                            (scope) => addAllTargets(scope, activeServerId)
                        "
                        @execute-copy="executeCopy"
                        @create-formation-variants="
                            showFormationVariationDialog = true
                        "
                        @set-group="
                            (id, value) => (copyGroupSelection[id] = value)
                        "
                    />
                </template>
            </main>
        </div>
    </TooltipProvider>
</template>
