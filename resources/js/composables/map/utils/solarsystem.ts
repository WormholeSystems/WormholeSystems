import { TMapSolarSystem } from '@/types/models';
import { mapState } from '../state';
import { TDataMapSolarSystem, WithIsSelected } from '../types';

export function getSelectedState(system: TMapSolarSystem): WithIsSelected<TMapSolarSystem> {
    if (!mapState.selection) return { ...system, is_selected: false };
    if (!mapState.selection.end) return { ...system, is_selected: false };

    const { start: raw_start, end: raw_end } = mapState.selection;

    const start = {
        x: raw_start.x / mapState.scale,
        y: raw_start.y / mapState.scale,
    };

    const end = {
        x: raw_end.x / mapState.scale,
        y: raw_end.y / mapState.scale,
    };

    const is_selected =
        system.position!.x >= Math.min(start.x, end.x) &&
        system.position!.x <= Math.max(start.x, end.x) &&
        system.position!.y >= Math.min(start.y, end.y) &&
        system.position!.y <= Math.max(start.y, end.y);

    return { ...system, is_selected };
}

export function getHoveredState(system: WithIsSelected<TMapSolarSystem>): TDataMapSolarSystem {
    const is_hovered = mapState.hovered_solarsystem_id === system.id;
    return { ...system, is_hovered };
}

export function applyScale(system: TDataMapSolarSystem): TDataMapSolarSystem {
    if (!system.position) return system;
    const scale = mapState.scale;
    return {
        ...system,
        position: {
            x: scale * system.position.x,
            y: scale * system.position.y,
        },
    };
}
