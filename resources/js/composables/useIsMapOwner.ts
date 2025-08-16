import { AppPageProps } from '@/types';
import { TMap } from '@/types/models';
import { usePage } from '@inertiajs/vue3';
import { computed } from 'vue';
import useUser from './useUser';

export default function useIsMapOwner() {
    const page = usePage<
        AppPageProps<{
            map: TMap;
        }>
    >();

    const user = useUser();

    return computed(() => {
        if (!user.value) return false;
        if (!page.props.map) {
            throw new Error('Map property not found in page props');
        }
        return user.value.characters.some((character) => character.id === page.props.map.owner.id);
    });
}
