import { isWormholeClass } from '@/const/solarsystemClasses';
import { TResolvedSolarsystem } from '@/pages/maps';

type BookmarkSolarsystem = Pick<TResolvedSolarsystem, 'class' | 'name'> & {
    region?: { name?: string | null } | null;
};

const KSPACE_BOOKMARK_LABELS: Record<string, string> = { h: 'HS', l: 'LS', n: 'NS' };

/** Compact ship-size labels. Large is the common case and intentionally omitted so the token only surfaces restrictive holes. */
const SHIP_SIZE_LABELS: Record<string, string> = { frigate: 'SM', medium: 'MD', xlarge: 'XM' };

/** Mass labels. Fresh/unknown intentionally resolve to nothing so the token drops out. */
const MASS_STATUS_LABELS: Record<string, string> = { reduced: 'reduced', critical: 'crit' };

/** Lifetime labels. Healthy intentionally resolves to nothing so the token drops out. Kept in the "EOL" vocabulary so it never collides with mass "crit". */
const LIFETIME_LABELS: Record<string, string> = { eol: 'EOL', critical: 'EOL!' };

type BookmarkSystem = {
    alias?: string | null;
    occupier_alias?: string | null;
    solarsystem: BookmarkSolarsystem;
};

/**
 * The normalized connection data a bookmark can reference. Every field is
 * optional: unidentified holes, unknown mass, gate connections and so on simply
 * leave the matching token empty so it drops out of the rendered name. Each call
 * site builds this from whatever signature/connection shape it has on hand.
 */
export type TBookmarkContext = {
    signatureId?: string | null;
    shipSize?: string | null;
    massStatus?: string | null;
    lifetime?: string | null;
    wormholeCode?: string | null;
};

/**
 * The placeholder tokens that may appear in a bookmark format template. Kept in
 * sync with the `BookmarkToken` enum on the backend.
 */
export const BOOKMARK_TOKENS = ['alias', 'sig', 'class', 'name', 'region', 'occupier', 'size', 'wh', 'mass', 'life'] as const;

export type TBookmarkToken = (typeof BOOKMARK_TOKENS)[number];

/** Default template for wormhole systems, e.g. "Home ABC C3". */
export const DEFAULT_BOOKMARK_FORMAT_WORMHOLE = '{alias} {sig} {class}';

/** Default template for k-space systems, e.g. "Home HS ABC Jita The Forge". */
export const DEFAULT_BOOKMARK_FORMAT_KSPACE = '{alias} {class} {sig} {name} {region}';

export type TBookmarkFormats = {
    bookmark_format_wormhole?: string | null;
    bookmark_format_kspace?: string | null;
};

/**
 * Short class label used in connection bookmarks: "C3" for wormhole systems,
 * otherwise "HS" / "LS" / "NS" derived from security.
 */
export function getBookmarkClassString(solarsystem: BookmarkSolarsystem): string {
    if (isWormholeClass(solarsystem.class)) return `C${solarsystem.class}`;
    return KSPACE_BOOKMARK_LABELS[solarsystem.class] ?? solarsystem.class.toUpperCase();
}

/**
 * The first three characters of a signature id (e.g. "ABC-123" -> "ABC"), or an
 * empty string when no signature is known.
 */
export function getSignatureIdShort(signatureId: string | null | undefined): string {
    return signatureId ? signatureId.substring(0, 3) : '';
}

/**
 * Resolve the value for every bookmark token for a given system and the
 * connection signature. Tokens with no value resolve to an empty string and are
 * dropped when the template renders. Mass and lifetime deliberately stay empty
 * while the hole is fresh/healthy, so they only surface once it degrades.
 */
export function getBookmarkTokenValues(system: BookmarkSystem, context: TBookmarkContext): Record<TBookmarkToken, string> {
    return {
        alias: system.alias ?? '',
        sig: getSignatureIdShort(context.signatureId),
        class: getBookmarkClassString(system.solarsystem),
        name: system.solarsystem.name,
        region: system.solarsystem.region?.name ?? '',
        occupier: system.occupier_alias ?? '',
        size: context.shipSize ? (SHIP_SIZE_LABELS[context.shipSize] ?? '') : '',
        wh: context.wormholeCode ?? '',
        mass: context.massStatus ? (MASS_STATUS_LABELS[context.massStatus] ?? '') : '',
        life: context.lifetime ? (LIFETIME_LABELS[context.lifetime] ?? '') : '',
    };
}

/**
 * Substitute `{token}` placeholders in a template, dropping tokens that resolve
 * to an empty value and collapsing the whitespace they leave behind. Unknown
 * placeholders are left untouched.
 */
export function renderBookmarkTemplate(template: string, values: Record<TBookmarkToken, string>): string {
    return template
        .replace(/\{(\w+)\}/g, (match, token: string) => (token in values ? values[token as TBookmarkToken] : match))
        .replace(/\s+/g, ' ')
        .trim();
}

/**
 * Build the connection bookmark name for a system using the map's configured
 * templates (falling back to the defaults). `context` carries the connection
 * data the template can reference (signature id, size, mass, lifetime, code).
 */
export function formatBookmarkName(system: BookmarkSystem, context: TBookmarkContext, formats?: TBookmarkFormats | null): string {
    const template = isWormholeClass(system.solarsystem.class)
        ? formats?.bookmark_format_wormhole || DEFAULT_BOOKMARK_FORMAT_WORMHOLE
        : formats?.bookmark_format_kspace || DEFAULT_BOOKMARK_FORMAT_KSPACE;

    return renderBookmarkTemplate(template, getBookmarkTokenValues(system, context));
}
