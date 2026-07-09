import type { ClassValue } from 'clsx'
import { clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs))
}

// Middle-truncates a filesystem path for display, keeping the root and the
// most specific segments: /Users/tim/…/settings_Default/core_user_1.dat
export function shortenPath(path: string, keepEnd = 2): string {
    const sep = path.includes('\\') ? '\\' : '/'
    const parts = path.split(sep)
    if (parts.length <= keepEnd + 3) return path
    return [...parts.slice(0, 3), '…', ...parts.slice(-keepEnd)].join(sep)
}
