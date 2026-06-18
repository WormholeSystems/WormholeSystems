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
import { useMapUserSettings } from '@/composables/useMapUserSettings';

export function guessNextAlias(parentAlias: string | null | undefined, aliases: string[]): string {
    const map_user_settings = useMapUserSettings();
    const concatDisabled = Boolean(map_user_settings.value?.concat_alias_disabled);

    const prefix = concatDisabled ? '' : (parentAlias ?? '').trim();

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

    const result = `${prefix}${highest + 1}`;
    return result.replace(/(\d{3})(?=\d)/g, '$1-');
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
}): string | null {
    const originIsAliased = Boolean(params.parentAlias && params.parentAlias.trim());

    if (!params.targetIsWormhole && !params.originIsWormhole && !originIsAliased) {
        return null;
    }

    return guessNextAlias(params.parentAlias, params.aliases);
}
