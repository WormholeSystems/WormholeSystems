type JoinWithBackslash<T extends string[]> = T extends [infer First extends string, ...infer Rest extends string[]]
    ? Rest['length'] extends 0
        ? First
        : `${First}\\${JoinWithBackslash<Rest>}`
    : '';

function getEventName<const T extends string[]>(...parts: T): JoinWithBackslash<T> {
    return parts.join('\\') as JoinWithBackslash<T>;
}

export const KillmailReceivedEvent = getEventName('Killmails', 'KillmailReceivedEvent');

export const MapSolarsystemsUpsertedEvent = getEventName('MapSolarsystems', 'MapSolarsystemsUpsertedEvent');
export const MapSolarsystemsRemovedEvent = getEventName('MapSolarsystems', 'MapSolarsystemsRemovedEvent');
export const MapConnectionsUpsertedEvent = getEventName('MapConnections', 'MapConnectionsUpsertedEvent');
export const MapConnectionsRemovedEvent = getEventName('MapConnections', 'MapConnectionsRemovedEvent');
export const MapMetadataUpdatedEvent = getEventName('Maps', 'MapMetadataUpdatedEvent');
export const MapResyncEvent = getEventName('Maps', 'MapResyncEvent');
export const SignaturesChangedEvent = getEventName('Signatures', 'SignaturesChangedEvent');

export const CharacterStatusUpdatedEvent = getEventName('Characters', 'CharacterStatusUpdatedEvent');
export const UserCharacterStatusUpdatedEvent = getEventName('Characters', 'UserCharacterStatusUpdatedEvent');
export const SignatureUpdatedEvent = getEventName('Signatures', 'SignatureUpdatedEvent');
export const SignatureCreatedEvent = getEventName('Signatures', 'SignatureCreatedEvent');
export const SignatureDeletedEvent = getEventName('Signatures', 'SignatureDeletedEvent');

export const MapRouteSolarsystemsUpdatedEvent = getEventName('MapRouteSolarsystems', 'MapRouteSolarsystemsUpdatedEvent');

export const MapIgnoredSolarsystemsUpdatedEvent = getEventName('MapIgnoredSolarsystems', 'MapIgnoredSolarsystemsUpdatedEvent');

export const ServerStatusUpdatedEvent = getEventName('ServerStatusUpdatedEvent');
