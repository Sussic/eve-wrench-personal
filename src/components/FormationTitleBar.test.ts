// @vitest-environment jsdom
import { mount } from '@vue/test-utils'
import { describe, expect, it, vi } from 'vitest'
import FormationTitleBar from './FormationTitleBar.vue'

vi.mock('@/composables/useI18n', () => ({
    useI18n: () => ({ t: (key: string) => key }),
}))

describe('FormationTitleBar', () => {
    it('keeps the controls outside drag regions and emits close', async () => {
        const wrapper = mount(FormationTitleBar, {
            props: {
                title: 'Probe Formations — 123',
                colorMode: 'dark',
                isMac: false,
                isMaximized: false,
            },
        })

        expect(wrapper.attributes('data-tauri-drag-region')).toBeUndefined()
        const controls = wrapper.get(
            '[data-testid="formation-window-controls"]'
        )
        expect(controls.attributes('data-tauri-drag-region')).toBeUndefined()
        expect(
            wrapper.findAll('[data-tauri-drag-region]').length
        ).toBeGreaterThan(0)

        await wrapper.get('button[title="common.close"]').trigger('click')
        expect(wrapper.emitted('close')).toHaveLength(1)
    })
})
