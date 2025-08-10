import { useLayout } from '@/composables/useLayout';
import { mapState, scale } from '../state';

export function useMapScale() {
    const { layout, setLayout } = useLayout();

    function setScale(newScale: number) {
        const roundedScale = Math.round(newScale * 10) / 10;
        if (roundedScale < 0.5 || roundedScale > 2) return;
        mapState.scale = roundedScale;
        setLayout({
            ...layout.value,
            scale: roundedScale,
        });
    }

    return {
        scale,
        setScale,
    };
}
