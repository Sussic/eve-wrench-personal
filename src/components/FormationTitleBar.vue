<script setup lang="ts">
import { Moon, Radar, Sun } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import WindowControls from '@/components/WindowControls.vue'
import { useI18n } from '@/composables/useI18n'

const { t } = useI18n()

defineProps<{
    title: string
    colorMode: string
    isMac: boolean
    isMaximized: boolean
}>()

const emit = defineEmits<{
    toggleTheme: []
    minimize: []
    toggleMaximize: []
    close: []
}>()
</script>

<template>
    <!-- Keep drag regions on dedicated elements. Putting one on the header
         causes Tauri to consume clicks intended for the window controls. -->
    <header
        class="flex h-11 shrink-0 items-center gap-2 border-b bg-background/80 px-3 backdrop-blur-sm"
    >
        <div data-tauri-drag-region class="h-full flex-1" />
        <div
            data-tauri-drag-region
            class="flex min-w-0 items-center gap-2 px-2"
        >
            <Radar
                data-tauri-drag-region
                class="size-4 shrink-0 text-foreground"
                :stroke-width="2"
            />
            <h1 data-tauri-drag-region class="truncate text-xs font-semibold">
                {{ title }}
            </h1>
        </div>
        <div
            data-testid="formation-window-controls"
            class="flex flex-1 items-center justify-end gap-1"
            @pointerdown.stop
            @mousedown.stop
        >
            <Button
                variant="ghost"
                size="icon"
                :title="t('titleBar.toggleTheme')"
                @click="emit('toggleTheme')"
            >
                <Sun v-if="colorMode === 'dark'" class="size-4" />
                <Moon v-else class="size-4" />
            </Button>
            <WindowControls
                v-if="!isMac"
                :is-maximized="isMaximized"
                @minimize="emit('minimize')"
                @toggle-maximize="emit('toggleMaximize')"
                @close="emit('close')"
            />
        </div>
    </header>
</template>
