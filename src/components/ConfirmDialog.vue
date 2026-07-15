<script setup lang="ts">
import {
    AlertDialog,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import { useConfirm } from '@/composables/useConfirm'

const { isOpen, options, handleConfirm, handleCancel } = useConfirm()
</script>

<template>
    <AlertDialog :open="isOpen">
        <AlertDialogContent>
            <AlertDialogHeader>
                <AlertDialogTitle>{{ options.title }}</AlertDialogTitle>
                <AlertDialogDescription class="whitespace-pre-line">{{
                    options.description
                }}</AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <Button variant="outline" @click="handleCancel">
                    {{ options.cancelText }}
                </Button>
                <Button
                    :variant="options.destructive ? 'destructive' : 'default'"
                    @click="handleConfirm"
                >
                    {{ options.confirmText }}
                </Button>
            </AlertDialogFooter>
        </AlertDialogContent>
    </AlertDialog>
</template>
