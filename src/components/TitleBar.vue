<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
    Wrench,
    Sun,
    Moon,
    RefreshCw,
    Settings,
    FolderOpen,
    RotateCcw,
    Download,
    Upload,
    Languages,
} from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
} from '@/components/ui/tooltip'
import {
    DropdownMenu,
    DropdownMenuCheckboxItem,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import WindowControls from '@/components/WindowControls.vue'
import { useWindowChrome } from '@/composables/useWindowChrome'
import { useI18n } from '@/composables/useI18n'

defineProps<{
    loading: boolean
    colorMode: string
    customEvePath: string | null
    autoBackup: boolean
}>()

const emit = defineEmits<{
    refresh: []
    toggleTheme: []
    selectEvePath: []
    clearEvePath: []
    exportSettings: []
    importSettings: []
    setAutoBackup: [enabled: boolean]
}>()

const { t, locale, languages, changeLanguage } = useI18n()
const { isMac, isMaximized, minimize, toggleMaximize, close } =
    useWindowChrome()
const appInfo = ref<{ version: string; preview: boolean } | null>(null)

onMounted(async () => {
    try {
        appInfo.value = await invoke<{ version: string; preview: boolean }>(
            'get_app_info'
        )
    } catch {
        appInfo.value = null
    }
})
</script>

<template>
    <header
        data-tauri-drag-region
        class="titlebar flex h-11 shrink-0 items-center justify-between border-b bg-background/80 px-4 backdrop-blur-sm"
    >
        <!-- Left: Spacer for macOS traffic lights -->
        <div class="w-20 shrink-0"></div>

        <!-- Center: Logo -->
        <div data-tauri-drag-region class="flex items-center gap-2">
            <Wrench class="size-4 text-primary" :stroke-width="2" />
            <span
                class="text-xs font-semibold tracking-widest text-muted-foreground"
            >
                EVE WRENCH
            </span>
            <span
                v-if="appInfo?.preview"
                class="rounded-full border border-amber-500/50 bg-amber-500/10 px-2 py-px text-[10px] font-semibold uppercase tracking-wider text-amber-600 dark:text-amber-400"
                :title="`v${appInfo.version}`"
            >
                {{ t('titleBar.preview') }}
            </span>
        </div>

        <!-- Right: Actions + Window controls -->
        <div class="flex shrink-0 items-center justify-end gap-1">
            <DropdownMenu>
                <DropdownMenuTrigger as-child>
                    <Button
                        variant="ghost"
                        size="icon"
                        :title="t('titleBar.settings')"
                    >
                        <Settings class="size-4" />
                    </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end" class="w-64">
                    <DropdownMenuLabel>{{
                        t('settings.eveSettingsFolder')
                    }}</DropdownMenuLabel>
                    <DropdownMenuItem @click="emit('selectEvePath')">
                        <FolderOpen class="mr-2 size-4" />
                        {{
                            customEvePath
                                ? t('settings.changeFolder')
                                : t('settings.setCustomPath')
                        }}
                    </DropdownMenuItem>
                    <template v-if="customEvePath">
                        <DropdownMenuSeparator />
                        <DropdownMenuLabel
                            class="font-normal text-xs text-muted-foreground truncate"
                        >
                            {{ customEvePath }}
                        </DropdownMenuLabel>
                        <DropdownMenuItem @click="emit('clearEvePath')">
                            <RotateCcw class="mr-2 size-4" />
                            {{ t('settings.resetToDefault') }}
                        </DropdownMenuItem>
                    </template>
                    <DropdownMenuSeparator />
                    <DropdownMenuLabel
                        >{{ t('importExport.import') }} /
                        {{ t('importExport.export') }}</DropdownMenuLabel
                    >
                    <DropdownMenuItem @click="emit('exportSettings')">
                        <Download class="mr-2 size-4" />
                        {{ t('importExport.exportSettings') }}
                    </DropdownMenuItem>
                    <DropdownMenuItem @click="emit('importSettings')">
                        <Upload class="mr-2 size-4" />
                        {{ t('importExport.importSettings') }}
                    </DropdownMenuItem>
                    <DropdownMenuSeparator />
                    <DropdownMenuLabel>{{
                        t('settings.backups')
                    }}</DropdownMenuLabel>
                    <DropdownMenuCheckboxItem
                        :model-value="autoBackup"
                        @update:model-value="
                            emit('setAutoBackup', $event === true)
                        "
                    >
                        {{ t('settings.autoBackup') }}
                    </DropdownMenuCheckboxItem>
                    <DropdownMenuSeparator />
                    <DropdownMenuLabel>{{
                        t('settings.language')
                    }}</DropdownMenuLabel>
                    <DropdownMenuItem
                        v-for="lang in languages"
                        :key="lang.code"
                        @click="changeLanguage(lang.code)"
                        :class="{ 'bg-muted': locale === lang.code }"
                    >
                        <Languages class="mr-2 size-4" />
                        {{ lang.name }}
                        <span v-if="locale === lang.code" class="ml-auto"
                            >✓</span
                        >
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>

            <Tooltip>
                <TooltipTrigger as-child>
                    <Button
                        variant="ghost"
                        size="icon"
                        @click="emit('toggleTheme')"
                    >
                        <Sun v-if="colorMode === 'dark'" class="size-4" />
                        <Moon v-else class="size-4" />
                    </Button>
                </TooltipTrigger>
                <TooltipContent>{{ t('titleBar.toggleTheme') }}</TooltipContent>
            </Tooltip>

            <Tooltip>
                <TooltipTrigger as-child>
                    <Button
                        variant="ghost"
                        size="icon"
                        :disabled="loading"
                        @click="emit('refresh')"
                    >
                        <RefreshCw
                            class="size-4"
                            :class="{ 'animate-spin': loading }"
                        />
                    </Button>
                </TooltipTrigger>
                <TooltipContent>{{ t('common.refresh') }}</TooltipContent>
            </Tooltip>

            <!-- Window controls (Windows/Linux only) -->
            <WindowControls
                v-if="!isMac"
                class="ml-2"
                :is-maximized="isMaximized"
                @minimize="minimize"
                @toggle-maximize="toggleMaximize"
                @close="close"
            />
        </div>
    </header>
</template>
