import { mapState, selectSystemsInBox, selection } from '../state';

export function useSelection() {
    function setSelectionStart(x: number, y: number) {
        mapState.selection = {
            start: { x, y },
            end: null,
        };
        mapState.selected_ids = [];
    }

    function setSelectionEnd(x: number, y: number) {
        if (mapState.selection) {
            mapState.selection.end = { x, y };
            selectSystemsInBox();
        }
    }

    function clearSelection() {
        mapState.selection = null;
        mapState.selected_ids = [];
    }

    return {
        selection,
        setSelectionStart,
        setSelectionEnd,
        clearSelection,
    };
}
