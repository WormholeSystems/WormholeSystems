import { TResolvedSolarsystem } from '@/pages/maps';

type BookmarkSolarsystem = Pick<TResolvedSolarsystem, 'class' | 'security' | 'name'> & {
    region?: { name?: string | null } | null;
};

type BookmarkSystem = {
    alias?: string | null;
    solarsystem: BookmarkSolarsystem;
};

/**
 * Short class label used in connection bookmarks: "C3" for wormhole systems,
 * otherwise "HS" / "LS" / "NS" derived from security.
 */
export function getBookmarkClassString(solarsystem: BookmarkSolarsystem): string {
    if (solarsystem.class) return `C${solarsystem.class}`;
    if (solarsystem.security >= 0.5) return 'HS';
    if (solarsystem.security > 0.0) return 'LS';
    return 'NS';
}

/**
 * The first three characters of a signature id (e.g. "ABC-123" -> "ABC"), or an
 * empty string when no signature is known.
 */
export function getSignatureIdShort(signatureId: string | null | undefined): string {
    return signatureId ? signatureId.substring(0, 3) : '';
}

/**
 * Build the connection bookmark name for a system, matching the scheme used in
 * the connection context menu. `signatureIdShort` is the short id of the
 * signature on the other side of the connection (empty when unknown).
 *
 * Wormhole systems read "alias sig class"; k-space systems read
 * "alias class sig name region".
 */
export function formatBookmarkName(system: BookmarkSystem, signatureIdShort: string): string {
    const class_string = getBookmarkClassString(system.solarsystem);

    if (system.solarsystem.class) {
        const parts = [system.alias || system.solarsystem.name];
        if (signatureIdShort) parts.push(signatureIdShort);
        parts.push(class_string);
        return parts.join(' ');
    }

    const parts: string[] = [];
    if (system.alias) parts.push(system.alias);
    parts.push(class_string);
    if (signatureIdShort) parts.push(signatureIdShort);
    parts.push(system.solarsystem.name);
    if (system.solarsystem.region?.name) parts.push(system.solarsystem.region.name);
    return parts.join(' ');
}
