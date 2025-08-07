import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

export function setCookie(name: string, value: string, days = 365) {
    if (typeof document === 'undefined') {
        return;
    }

    const maxAge = days * 24 * 60 * 60;

    document.cookie = `${name}=${value};path=/;max-age=${maxAge};SameSite=Lax`;
}

const compactNumberFormat = new Intl.NumberFormat('en-US', {
    notation: 'compact',
});

export function formatISK(value: number): string {
    if (value === 0) return '0 ISK';

    const formattedValue = compactNumberFormat.format(value);
    return `${formattedValue} ISK`;
}
