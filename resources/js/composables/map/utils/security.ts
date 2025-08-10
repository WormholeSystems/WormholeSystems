export function getSecurityClass(security: number): string {
    if (security >= 0.5) return 'high';
    if (security >= 0.1) return 'low';
    return 'null';
}
