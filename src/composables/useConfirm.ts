import { ref } from 'vue'
import { useI18n } from './useI18n'

interface ConfirmOptions {
    title?: string
    description: string
    confirmText?: string
    cancelText?: string
    destructive?: boolean
}

const isOpen = ref(false)
const options = ref<ConfirmOptions>({ description: '' })
let resolvePromise: ((value: boolean) => void) | null = null

export function useConfirm() {
    const { t } = useI18n()

    function confirm(opts: ConfirmOptions | string): Promise<boolean> {
        if (typeof opts === 'string') {
            opts = { description: opts }
        }
        options.value = {
            title: opts.title || t('dialog.confirm'),
            description: opts.description,
            confirmText: opts.confirmText || t('dialog.continue'),
            cancelText: opts.cancelText || t('dialog.cancel'),
            destructive: opts.destructive || false,
        }
        isOpen.value = true

        return new Promise((resolve) => {
            resolvePromise = resolve
        })
    }

    function handleConfirm() {
        if (resolvePromise) {
            resolvePromise(true)
            resolvePromise = null
        }
        isOpen.value = false
    }

    function handleCancel() {
        if (resolvePromise) {
            resolvePromise(false)
            resolvePromise = null
        }
        isOpen.value = false
    }

    return {
        isOpen,
        options,
        confirm,
        handleConfirm,
        handleCancel,
    }
}
