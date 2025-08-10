import MapSelection from '@/routes/map-selection';
import { router } from '@inertiajs/vue3';
import { mapState, map_solarsystems_selected } from '../state';
import { item_height, sortByAlias, sortByClass, sortByName, sortByRegion } from '../utils';

function calculateSortedPositions(sorted_ids: number[]) {
    const sorted_positions = map_solarsystems_selected.value
        .sort((a, b) => {
            if (a.position && b.position) {
                return a.position.x - b.position.x || a.position.y - b.position.y;
            }
            return 0;
        })
        .map((s) => ({
            ...s,
            position_x: s.position?.x,
            position_y: s.position?.y,
        }));

    return sorted_ids.map((id, index) => ({
        ...sorted_positions[index],
        id,
        position_x: sorted_positions[index].position_x,
        position_y: sorted_positions[index].position_y,
    }));
}

export function organizeMapSolarsystems(spacing: number = 1) {
    const first_position = map_solarsystems_selected.value.reduce(
        (acc, system) => {
            if (!system.position) return acc;
            return {
                x: Math.min(acc.x, system.position.x),
                y: Math.min(acc.y, system.position.y),
            };
        },
        { x: Infinity, y: Infinity },
    );

    const sorted_ids = map_solarsystems_selected.value
        .toSorted(sortByName)
        .toSorted(sortByRegion)
        .toSorted(sortByClass)
        .toSorted(sortByAlias)
        .map((s) => s.id);

    const sorted_positions = calculateSortedPositions(sorted_ids).map((s, index) => {
        const position_x = first_position.x;
        const position_y = first_position.y + index * (spacing * mapState.config.grid_size + item_height) * mapState.scale;

        return {
            ...s,
            position_x: Math.max(40, Math.min(position_x, mapState.config.max_size.x - mapState.config.grid_size)) / mapState.scale,
            position_y: Math.max(20, Math.min(position_y, mapState.config.max_size.y - mapState.config.grid_size)) / mapState.scale,
        };
    });

    router.put(
        MapSelection.update().url,
        {
            map_solarsystems: sorted_positions,
        },
        {
            preserveState: true,
            preserveScroll: true,
            onSuccess: () => {
                mapState.selection = null;
            },
            only: ['map'],
        },
    );
}
