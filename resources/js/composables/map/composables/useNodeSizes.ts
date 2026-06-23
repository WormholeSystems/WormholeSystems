import { reactive } from 'vue';

export type NodeSize = {
    /** Width in base (unscaled) units. */
    width: number;
    /** Height in base (unscaled) units. */
    height: number;
};

// Measured per map-solarsystem so connections can attach to a node's real edge
// regardless of view. Nodes are content-sized in the manual layout, so the size
// is not known up front and has to be reported by each rendered node.
const nodeSizes = reactive(new Map<number, NodeSize>());

export function reportNodeSize(id: number, size: NodeSize): void {
    nodeSizes.set(id, size);
}

export function forgetNodeSize(id: number): void {
    nodeSizes.delete(id);
}

export function useNodeSizes() {
    return { nodeSizes };
}
