/**
 * Work out the next concatenated child alias for a system, given its parent's
 * alias and every alias already in use on the map.
 *
 * Top-level systems (no parent alias) are numbered 1, 2, 3…; children of "1"
 * become 11, 12, 13…; children of "12" become 121, 122… The next index is the
 * highest existing direct-child index + 1. Direct children are aliases that
 * extend the parent's prefix with digits and are not themselves nested under a
 * longer prefix, so "121" is never mistaken for a direct child of "1".
 */
export function guessNextAlias(parentAlias: string | null | undefined, aliases: string[]): string {
    const prefix = (parentAlias ?? '').trim();

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
 * should not be aliased. K-space targets are only aliased when the map's home
 * system is a wormhole; wormhole targets are always aliased.
 */
export function suggestAlias(params: {
    parentAlias: string | null | undefined;
    targetIsWormhole: boolean;
    homeIsWormhole: boolean;
    aliases: string[];
}): string | null {
    if (!params.targetIsWormhole && !params.homeIsWormhole) return null;

    return guessNextAlias(params.parentAlias, params.aliases);
}
