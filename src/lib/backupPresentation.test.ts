import { describe, expect, it } from 'vitest'
import { backupReasonKey } from './backupPresentation'

describe('backup presentation', () => {
    it('turns internal automatic labels into readable reason keys', () => {
        expect(backupReasonKey('pre-selective-copy')).toBe('preSelectiveCopy')
        expect(backupReasonKey('pre-formation-variants (2)')).toBe(
            'preFormationVariants'
        )
        expect(backupReasonKey('My manual backup')).toBeNull()
    })
})
