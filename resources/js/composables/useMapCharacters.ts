import { AppPageProps } from '@/types';
import { TCharacter } from '@/types/models';
import { usePage } from '@inertiajs/vue3';
import { computed } from 'vue';

export function useMapCharacters() {
    const page = usePage<
        AppPageProps<{
            map_characters: TCharacter[] | null;
        }>
    >();

    return computed(() => {
        return page.props.map_characters || [];
    });
}
