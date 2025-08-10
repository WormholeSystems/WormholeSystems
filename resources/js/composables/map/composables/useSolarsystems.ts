import { TMapSolarSystem } from '@/types/models';
import { mapState, map_solarsystems, map_solarsystems_selected } from '../state';

export function useMapSolarsystems() {
    function setSystemPosition(system: TMapSolarSystem, raw_x: number, raw_y: number) {
        const system_state = mapState.map_solarsystems.find((s) => s.id === system.id);
        if (!system_state) return;
        const x = raw_x;
        const y = raw_y;
        const dx = x - system_state.position!.x;
        const dy = y - system_state.position!.y;
        if (!map_solarsystems_selected.value.length || !map_solarsystems_selected.value.some((s) => s.id === system.id)) {
            if (system_state?.pinned) return;
            system_state.position = { x, y };
            mapState.selection = null;
            return;
        }

        mapState.map_solarsystems.forEach((s) => {
            if (s.is_selected && !s.pinned) {
                s.position = {
                    x: s.position!.x + dx,
                    y: s.position!.y + dy,
                };
            }
        });
    }

    function setHoveredMapSolarsystem(map_solarsystem_id: number, is_hovered: boolean) {
        if (is_hovered) {
            mapState.hovered_solarsystem_id = map_solarsystem_id;
        } else {
            mapState.hovered_solarsystem_id = null;
        }
    }

    return {
        map_solarsystems,
        map_solarsystems_selected,
        setSystemPosition,
        setHoveredMapSolarsystem,
    };
}
