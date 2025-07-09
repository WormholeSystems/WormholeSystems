type JoinWithBackslash<T extends string[]> = T extends [infer First extends string, ...infer Rest extends string[]]
    ? Rest['length'] extends 0
        ? First
        : `${First}\\${JoinWithBackslash<Rest>}`
    : '';

function getEventName<const T extends string[]>(...parts: T): JoinWithBackslash<T> {
    return parts.join('\\') as JoinWithBackslash<T>;
}

export const KillmailReceivedEvent = getEventName('Killmails', 'KillmailReceivedEvent');
export const MapUpdatedEvent = getEventName('Maps', 'MapUpdatedEvent');

export const MapSolarsystemCreatedEvent = getEventName('MapSolarsystems', 'MapSolarsystemCreatedEvent');
export const MapSolarsystemUpdatedEvent = getEventName('MapSolarsystems', 'MapSolarsystemUpdatedEvent');
export const MapSolarsystemDeletedEvent = getEventName('MapSolarsystems', 'MapSolarsystemDeletedEvent');
export const MapSolarsystemsUpdatedEvent = getEventName('MapSolarsystems', 'MapSolarsystemsUpdatedEvent');
export const MapSolarsystemsDeletedEvent = getEventName('MapSolarsystems', 'MapSolarsystemsDeletedEvent');

export const MapConnectionCreatedEvent = getEventName('MapConnections', 'MapConnectionCreatedEvent');
export const MapConnectionUpdatedEvent = getEventName('MapConnections', 'MapConnectionUpdatedEvent');
export const MapConnectionDeletedEvent = getEventName('MapConnections', 'MapConnectionDeletedEvent');

export const CharacterStatusUpdatedEvent = getEventName('Characters', 'CharacterStatusUpdatedEvent');
export const SignaturesUpdatedEvent = getEventName('Signatures', 'SignaturesUpdatedEvent');
