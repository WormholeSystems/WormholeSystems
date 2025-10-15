import { TKSpaceClass } from '@/types/models';

export function getSecurityClass(security: number): TKSpaceClass {
    if (security >= 0.5) return 'h';
    if (security >= 0.1) return 'l';
    return 'n';
}
