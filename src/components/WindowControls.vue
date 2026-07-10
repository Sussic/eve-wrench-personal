<script setup lang="ts">
import { Minus, Square, Copy, X } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { useI18n } from '@/composables/useI18n'

const { t } = useI18n()

defineProps<{
    isMaximized: boolean
}>()

const emit = defineEmits<{
    minimize: []
    toggleMaximize: []
    close: []
}>()
</script>

<template>
    <div class="flex">
        <Button
            variant="window"
            size="icon"
            title="Minimize"
            @click="emit('minimize')"
        >
            <Minus class="size-4" :stroke-width="1.5" />
        </Button>
        <Button
            variant="window"
            size="icon"
            :title="isMaximized ? 'Restore' : 'Maximize'"
            @click="emit('toggleMaximize')"
        >
            <Copy v-if="isMaximized" class="size-3.5" :stroke-width="1.5" />
            <Square v-else class="size-3" :stroke-width="1.5" />
        </Button>
        <Button
            variant="windowClose"
            size="icon"
            :title="t('common.close')"
            @click="emit('close')"
        >
            <X class="size-4" :stroke-width="1.5" />
        </Button>
    </div>
</template>
