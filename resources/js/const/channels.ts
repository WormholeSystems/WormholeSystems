export function getMapChannelName(map_id: number): string {
    return `Map.${map_id}`;
}

export function getServerStatusChannelName(): string {
    return 'ServerStatus';
}

export function getUserChannelName(user_id: number): string {
    return `User.${user_id}`;
}
