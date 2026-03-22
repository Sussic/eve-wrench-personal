<script setup lang="ts">
import 'vue-sonner/style.css'
import { onMounted } from 'vue'
import { useColorMode } from '@vueuse/core'
import { Toaster } from '@/components/ui/sonner'
import { TooltipProvider } from '@/components/ui/tooltip'
import { Rocket, FolderOpen } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import TitleBar from '@/components/TitleBar.vue'
import SettingsBrowser from '@/components/SettingsBrowser.vue'
import CopyPanel from '@/components/CopyPanel.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import PromptDialog from '@/components/PromptDialog.vue'
import UpdateModal from '@/components/UpdateModal.vue'
import ImportDialog from '@/components/ImportDialog.vue'
import { useCopyManager } from '@/composables/useCopyManager'
import { useUpdateChecker } from '@/composables/useUpdateChecker'
import { useI18n } from '@/composables/useI18n'
import { isBackup } from '@/types'

const colorMode = useColorMode()
const { t } = useI18n()
const { updateAvailable, updateInfo, dismissed, dismiss } = useUpdateChecker()

const {
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
} = useCopyManager()

function isBackupSource(backup: { id: string }): boolean {
    return !!(source.value && isBackup(source.value) && source.value.id === backup.id)
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
            <UpdateModal
                v-if="updateAvailable && updateInfo && !dismissed"
                :info="updateInfo"
                @dismiss="dismiss"
            />

            <TitleBar
                :loading="loading"
                :color-mode="colorMode"
                :custom-eve-path="customEvePath"
                @refresh="refresh"
                @toggle-theme="toggleDarkMode"
                @select-eve-path="selectCustomEvePath"
                @clear-eve-path="clearCustomEvePath"
                @export-settings="exportSettings"
                @import-settings="importSettings"
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
                        @backup="createBackup"
                        @restore="restoreBackup"
                        @apply-backup="applyBackup"
                        @add-all-from-profile="addAllFromProfile"
                        @set-backup-source="setSource"
                        @delete-backup="deleteBackup"
                        @refresh="refresh"
                        @set-brackets-always-show="setBracketsAlwaysShow"
                    />

                    <CopyPanel
                        :source="source"
                        :targets="targets"
                        :can-copy="canCopy"
                        :copying="copying"
                        @clear-source="clearSource"
                        @remove-target="removeTarget"
                        @clear-targets="clearTargets"
                        @execute-copy="executeCopy"
                    />
                </template>
            </main>
        </div>
    </TooltipProvider>
</template>
