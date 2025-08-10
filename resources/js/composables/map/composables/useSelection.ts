import { mapState, selection } from '../state';

export function useSelection() {
    function setSelectionStart(x: number, y: number) {
        mapState.selection = {
            start: { x, y },
            end: null,
        };
    }

    function setSelectionEnd(x: number, y: number) {
        if (mapState.selection) {
            mapState.selection.end = { x, y };
        }
    }

    function clearSelection() {
        mapState.selection = null;
    }

    return {
        selection,
        setSelectionStart,
        setSelectionEnd,
        clearSelection,
    };
}
