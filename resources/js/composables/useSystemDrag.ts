import { TMapSolarSystem } from '@/types/models';
import { computed, reactive } from 'vue';

type TStore =
    | {
          position: { x: number; y: number };
          dragged_solarsystem: TMapSolarSystem;
      }
    | {
          position: null;
          dragged_solarsystem: null;
      };

const store = reactive<TStore>({
    position: null,
    dragged_solarsystem: null,
});

export function useSystemDrag() {
    function updateDragPosition(
        position: {
            x: number;
            y: number;
        } | null,
        dragged_solarsystem: TMapSolarSystem | null,
    ) {
        store.position = position;
        store.dragged_solarsystem = dragged_solarsystem;
    }

    const position = computed(() => store.position);
    const dragged_solarsystem = computed(() => store.dragged_solarsystem);

    return {
        position,
        dragged_solarsystem,
        updateDragPosition,
    };
}
