import { readonly, ref } from 'vue';

const fromSystemId = ref<number | null>(null);
const toSystemId = ref<number | null>(null);

export function useNavigationSystems() {
    function setFromSystem(solarsystemId: number): void {
        fromSystemId.value = solarsystemId;
    }

    function setToSystem(solarsystemId: number): void {
        toSystemId.value = solarsystemId;
    }

    function clearFromSystem(): void {
        fromSystemId.value = null;
    }

    function clearToSystem(): void {
        toSystemId.value = null;
    }

    function swapSystems(): void {
        const temp = fromSystemId.value;
        fromSystemId.value = toSystemId.value;
        toSystemId.value = temp;
    }

    return {
        fromSystemId: readonly(fromSystemId),
        toSystemId: readonly(toSystemId),
        setFromSystem,
        setToSystem,
        clearFromSystem,
        clearToSystem,
        swapSystems,
    };
}
