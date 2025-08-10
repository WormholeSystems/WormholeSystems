import { grid_size, mapState } from '../state';

export function useMapGrid() {
    function setMapGridSize(size: number) {
        mapState.config.grid_size = size;
    }

    return {
        grid_size,
        setMapGridSize,
    };
}
