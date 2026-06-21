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

const FIRST_LAYER_NATO_ALIASES = [
    'ALPHA',
    'BRAVO',
    'DELTA',
    'ECHO',
    'FOXTROT',
    'GOLF',
    'HOTEL',
    'INDIA',
    'JULIETT',
    'KILO',
    'LIMA',
    'MIKE',
    'NOVEMBER',
    'OSCAR',
    'PAPA',
    'QUEBEC',
    'ROMEO',
    'SIERRA',
    'TANGO',
    'UNIFORM',
    'VICTOR',
    'WHISKEY',
    'X-RAY',
    'YANKEE',
    'ZULU',
];

function nextNatoAlias(aliases: string[]): string {
    const existingWords = new Set(
        aliases
            .map((alias) => alias.trim())
            .filter((alias) => FIRST_LAYER_NATO_ALIASES.includes(alias)),
    );

    return FIRST_LAYER_NATO_ALIASES.find((word) => !existingWords.has(word)) ?? 'ZULU';
}

function isTopLevelAlias(parentAlias: string | null | undefined): boolean {
    return !parentAlias || !parentAlias.trim();
}

function isNatoAlias(alias: string | null | undefined): boolean {
    if (!alias) return false;
    return FIRST_LAYER_NATO_ALIASES.includes(alias.trim());
}

function nextNatoChildAlias(parentAlias: string, aliases: string[]): string {
    const letter = parentAlias.trim().charAt(0).toUpperCase();
    // Support existing aliases that use hyphen separators (e.g. A123-1)
    const childPattern = new RegExp(`^${letter}([0-9-]+)$`);

    const numericChildren = aliases
        .map((a) => a.trim().toUpperCase())
        .filter((alias) => {
            const match = alias.match(childPattern);
            return Boolean(match);
        });

    const highest = numericChildren.reduce((max, alias) => {
        const match = alias.match(childPattern);
        if (!match) return max;
        // Remove hyphens before parsing the numeric index
        const digits = match[1].replace(/-/g, '');
        const index = Number.parseInt(digits, 10);
        return Number.isNaN(index) ? max : Math.max(max, index);
    }, 0);

    return `${letter}${highest + 1}`;
}

export function guessNextAlias(
    parentAlias: string | null | undefined,
    aliases: string[],
    options: { targetIsWormhole?: boolean; originIsWormhole?: boolean; connectedAliases?: Array<string | null | undefined> } = {},
): string {
    const { targetIsWormhole = false, originIsWormhole = false, connectedAliases = [] } = options;
    const map_user_settings = useMapUserSettings();
    const concatDisabled = Boolean(map_user_settings.value?.concat_alias_disabled);
    const firstLayerNatoAlias = Boolean(map_user_settings.value?.first_layer_nato_alias);

    const prefix = concatDisabled ? '' : (parentAlias ?? '').trim();
    const normalizedPrefix = prefix.replace(/-/g, '');

    if (firstLayerNatoAlias && isTopLevelAlias(parentAlias)) {
        return nextNatoAlias(aliases);
    }

    if (firstLayerNatoAlias && isNatoAlias(parentAlias ?? null)) {
        const numericChildren = aliases.filter((alias) => {
            const match = alias.trim().toUpperCase().match(new RegExp(`^${parentAlias!.trim().charAt(0).toUpperCase()}(\\d+)$`));
            return Boolean(match);
        });

        if (originIsWormhole && targetIsWormhole && numericChildren.length === 0) {
            return parentAlias!.trim();
        }

        return nextNatoChildAlias(parentAlias!.trim(), aliases);
    }

    if (concatDisabled && parentAlias) {
        const normalizedParentAlias = parentAlias.trim().replace(/-/g, '');
        const normalizedConnectedAliases = connectedAliases.map((alias) => alias?.trim().replace(/-/g, '') ?? '');
        const numericConnectedAliases = normalizedConnectedAliases
            .filter((alias) => /^\d+$/.test(alias))
            .map((alias) => Number.parseInt(alias, 10));
        const highestConnectedNumericAlias = numericConnectedAliases.reduce((max, alias) => Math.max(max, alias), 0);

        if (/^\d+$/.test(normalizedParentAlias)) {
            const parentIndex = Number.parseInt(normalizedParentAlias, 10);
            const sameAliasConnections = normalizedConnectedAliases.filter((alias) => alias === normalizedParentAlias).length;
            const hasUnnamedOrNonNumericConnection = normalizedConnectedAliases.some((alias) => alias === '' || !/^\d+$/.test(alias));

            if (sameAliasConnections === 0 || (sameAliasConnections === 1 && !hasUnnamedOrNonNumericConnection)) {
                return parentAlias.trim();
            }

            return `${Math.max(highestConnectedNumericAlias, parentIndex) + 1}`.replace(/(\d{3})(?=\d)/g, '$1-');
        }

        return `${highestConnectedNumericAlias + 1}`.replace(/(\d{3})(?=\d)/g, '$1-');
    }

    const normalizedParentAlias = concatDisabled && parentAlias ? parentAlias.trim().replace(/-/g, '') : null;

    const numericChildren = aliases.filter((alias) => {
        const normalized = alias.trim().replace(/-/g, '');

        if (concatDisabled && normalizedParentAlias) {
            if (normalized.length <= normalizedParentAlias.length) return false;
            if (!normalized.startsWith(normalizedParentAlias)) return false;
            return /^\d+$/.test(normalized.slice(normalizedParentAlias.length));
        }

        if (normalized.length <= normalizedPrefix.length) return false;
        if (!normalized.startsWith(normalizedPrefix)) return false;
        if (normalizedParentAlias && normalized === normalizedParentAlias) return false;
        return /^\d+$/.test(normalized.slice(normalizedPrefix.length));
    });

    const directChildren = numericChildren.filter((alias) => {
        const normalized = alias.trim().replace(/-/g, '');
        return !numericChildren.some((other) => {
            const on = other.trim().replace(/-/g, '');
            return other !== alias && on.length < normalized.length && normalized.startsWith(on);
        });
    });

    if (originIsWormhole && targetIsWormhole && directChildren.length === 0 && parentAlias) {
        return parentAlias.trim();
    }

    const highest = directChildren.reduce((max, alias) => {
        const normalized = alias.trim().replace(/-/g, '');
        const index = Number.parseInt(normalized.slice(normalizedPrefix.length), 10);
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
    connectedAliases?: Array<string | null | undefined>;
}): string | null {
    const originIsAliased = Boolean(params.parentAlias && params.parentAlias.trim());

    if (!params.targetIsWormhole && !params.originIsWormhole && !originIsAliased) {
        return null;
    }

    return guessNextAlias(params.parentAlias, params.aliases, {
        targetIsWormhole: params.targetIsWormhole,
        originIsWormhole: params.originIsWormhole,
        connectedAliases: params.connectedAliases,
    });
}
