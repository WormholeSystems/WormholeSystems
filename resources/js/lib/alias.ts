/**
 * The per-map alias suggestion convention. Kept in sync with the `AliasScheme`
 * enum on the backend.
 */
export type TAliasScheme = 'numeric' | 'alphabetical';

/**
 * The kind of system an alphabetical suggestion is being generated for. K-space
 * targets get a reserved letter (H/L/N/P) instead of continuing the plain
 * letter sequence.
 */
export type TAliasTargetKind = 'wormhole' | 'h' | 'l' | 'n' | 'p';

type TGuessNextAliasOptions = {
    scheme?: TAliasScheme;
    targetKind?: TAliasTargetKind;
};

const KSPACE_ALIAS_TARGET_KINDS: readonly string[] = ['h', 'l', 'n', 'p'];

/** The reserved k-space letter for a target's class, or undefined for wormholes/unrecognized classes. */
export function aliasTargetKind(isTargetWormhole: boolean, targetClass: string | null | undefined): TAliasTargetKind | undefined {
    if (isTargetWormhole) return 'wormhole';
    return targetClass && KSPACE_ALIAS_TARGET_KINDS.includes(targetClass) ? (targetClass as TAliasTargetKind) : undefined;
}

/**
 * The alphabetical scheme's 22-letter alphabet: A-Z with H, L, N and P removed,
 * since those are reserved for k-space exits (high/low/null-sec, Pochven).
 */
const WORMHOLE_LETTERS = 'ABCDEFGIJKMOQRSTUVWXYZ';

/**
 * The next letter after `index`, capped at the last letter once the 22-letter
 * alphabet is exhausted. This mirrors numeric's unhandled ">9 children"
 * ambiguity rather than throwing: a map with more than 22 direct wormhole
 * children of one system will see duplicate suggestions past that point.
 */
function letterAtIndex(index: number): string {
    return WORMHOLE_LETTERS[Math.min(index, WORMHOLE_LETTERS.length - 1)];
}

/**
 * The next unused letter extending `prefix`, skipping reserved k-space
 * letters. Only aliases that are exactly one letter longer than `prefix`
 * count as direct children, so k-space exits (`AH1`) and deeper descendants
 * (`ABA`) are naturally excluded.
 */
function nextWormholeLetter(prefix: string, aliases: string[]): string {
    const highest = aliases.reduce((max, alias) => {
        if (alias.length !== prefix.length + 1 || !alias.startsWith(prefix)) return max;

        const index = WORMHOLE_LETTERS.indexOf(alias.slice(prefix.length).toUpperCase());
        return index === -1 ? max : Math.max(max, index);
    }, -1);

    return letterAtIndex(highest + 1);
}

/**
 * The next unused per-type index for a k-space exit, e.g. `AH1`, `AH2` for
 * high-sec children of `A`. Each reserved letter keeps its own counter, and
 * the anchored digit match excludes anything branching further off a k-space
 * node (`AH1A` is not counted as an `AH` index).
 */
function nextKspaceIndex(prefix: string, letter: string, aliases: string[]): number {
    const marker = `${prefix}${letter}`;

    const highest = aliases.reduce((max, alias) => {
        if (!alias.startsWith(marker)) return max;

        const tail = alias.slice(marker.length);
        return /^\d+$/.test(tail) ? Math.max(max, Number.parseInt(tail, 10)) : max;
    }, 0);

    return highest + 1;
}

/**
 * The alphabetical counterpart to the numeric digit logic: wormhole children
 * extend the prefix with a single non-reserved letter, k-space exits extend it
 * with a reserved letter and a per-type index.
 */
function guessNextAlphabeticalAlias(prefix: string, aliases: string[], targetKind: TAliasTargetKind | undefined): string {
    if (targetKind && targetKind !== 'wormhole') {
        const letter = targetKind.toUpperCase();
        return `${prefix}${letter}${nextKspaceIndex(prefix, letter, aliases)}`;
    }

    return `${prefix}${nextWormholeLetter(prefix, aliases)}`;
}

/**
 * Work out the next concatenated child alias for a system, given its parent's
 * alias and every alias already in use on the map.
 *
 * Numeric (default): top-level systems (no parent alias) are numbered 1, 2,
 * 3…; children of "1" become 11, 12, 13…; children of "12" become 121, 122…
 * The next index is the highest existing direct-child index + 1. Direct
 * children are aliases that extend the parent's prefix with digits and are
 * not themselves nested under a longer prefix, so "121" is never mistaken for
 * a direct child of "1".
 *
 * Alphabetical (`opts.scheme`): children use letters instead of digits (see
 * `guessNextAlphabeticalAlias`).
 */
export function guessNextAlias(parentAlias: string | null | undefined, aliases: string[], opts?: TGuessNextAliasOptions): string {
    const prefix = (parentAlias ?? '').trim();

    if (opts?.scheme === 'alphabetical') {
        return guessNextAlphabeticalAlias(prefix, aliases, opts.targetKind);
    }

    const numericChildren = aliases.filter((alias) => {
        if (alias.length <= prefix.length) return false;
        if (!alias.startsWith(prefix)) return false;
        return /^\d+$/.test(alias.slice(prefix.length));
    });

    const directChildren = numericChildren.filter(
        (alias) => !numericChildren.some((other) => other !== alias && other.length < alias.length && alias.startsWith(other)),
    );

    const highest = directChildren.reduce((max, alias) => {
        const index = Number.parseInt(alias.slice(prefix.length), 10);
        return Number.isNaN(index) ? max : Math.max(max, index);
    }, 0);

    return `${prefix}${highest + 1}`;
}

/**
 * Suggest an alias for a system reached by a tracked jump, or null when it
 * should not be aliased. The target is aliased when it is itself a wormhole, or
 * when the origin we jumped from is part of the chain — either a wormhole or an
 * already-aliased system. This lets a k-space exit of an aliased wormhole
 * continue the chain (e.g. jumping from "2" into k-space suggests "21").
 */
export function suggestAlias(params: {
    parentAlias: string | null | undefined;
    targetIsWormhole: boolean;
    originIsWormhole: boolean;
    aliases: string[];
    scheme?: TAliasScheme;
    targetKind?: TAliasTargetKind;
}): string | null {
    const originIsAliased = Boolean(params.parentAlias && params.parentAlias.trim());

    if (!params.targetIsWormhole && !params.originIsWormhole && !originIsAliased) {
        return null;
    }

    return guessNextAlias(params.parentAlias, params.aliases, { scheme: params.scheme, targetKind: params.targetKind });
}
