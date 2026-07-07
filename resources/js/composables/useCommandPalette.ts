import { ref } from 'vue';

const open = ref(false);

/** Shared so the status-bar button and the Cmd/Ctrl+K shortcut drive the same dialog. */
export function useCommandPalette() {
    function openPalette(): void {
        open.value = true;
    }

    return { open, openPalette };
}
