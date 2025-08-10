import { mapState, map_solarsystems } from '../state';
import { Coordinates } from '../types';

export const item_height = 40;

export function getFreePosition(): Coordinates {
    const map_width = mapState.config.max_size.x;
    const map_height = mapState.config.max_size.y;
    const padding = 100; // Padding to avoid edges
    const grid_size = mapState.config.grid_size;
    const boundary_box = {
        x1: -30,
        y1: -30,
        x2: 80,
        y2: 30,
    }; // Relative boundary box for the position of the solar system

    let x = padding;
    let y = padding;

    while (x < map_width - padding) {
        while (y < map_height - padding) {
            const overlaps = map_solarsystems.value.some((s) => {
                const position = { x, y };
                const system_boundary_box = {
                    x1: position.x + boundary_box.x1,
                    y1: position.y + boundary_box.y1,
                    x2: position.x + boundary_box.x2,
                    y2: position.y + boundary_box.y2,
                };

                return (
                    s.position &&
                    s.position.x >= system_boundary_box.x1 &&
                    s.position.x <= system_boundary_box.x2 &&
                    s.position.y >= system_boundary_box.y1 &&
                    s.position.y <= system_boundary_box.y2
                );
            });

            if (!overlaps) {
                return { x, y };
            }

            y += grid_size;
        }

        y = padding;
        x += grid_size;
    }

    throw new Error('No free position found on the map');
}
