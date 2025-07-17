<script setup lang="ts">
import CheckIcon from '@/components/icons/CheckIcon.vue';
import TrashIcon from '@/components/icons/TrashIcon.vue';
import { Button } from '@/components/ui/button';
import { PinInput, PinInputGroup, PinInputSeparator, PinInputSlot } from '@/components/ui/pin-input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { TableCell, TableRow } from '@/components/ui/table';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import signature_tree from '@/const/signatures';
import { TShowMapProps } from '@/pages/maps';
import { AppPageProps } from '@/types';
import { TSignature } from '@/types/models';
import { useForm, usePage } from '@inertiajs/vue3';
import { useNow } from '@vueuse/core';
import { formatDistanceStrict } from 'date-fns';
import { computed, watch } from 'vue';

const { signature } = defineProps<{
    signature: TSignature;
    deleted?: boolean;
    new?: boolean;
    updated?: boolean;
}>();

const form = useForm(() => ({
    signature_id: signature.signature_id || '',
    category: signature.category,
    type: signature.type,
}));

const now = useNow();

const wh_class = computed(() => {
    return (page.props.selected_map_solarsystem?.class as keyof typeof signature_tree.wormhole_space) || null;
});

const security = computed(() => {
    return page.props.selected_map_solarsystem?.solarsystem?.security || 0;
});

const modified_at = computed(() => {
    const modified = new Date(signature.updated_at || signature.created_at);
    return formatDistanceStrict(modified, now.value, {
        addSuffix: true,
    });
});

const page = usePage<AppPageProps<TShowMapProps>>();
const options = ['Wormhole', 'Gas Site', 'Ore Site', 'Combat Site', 'Data Site', 'Relic Site', 'Unknown'];

const signature_options = computed(() => {
    if (form.category === null) {
        return [];
    }
    if (wh_class.value) {
        const class_data = signature_tree.wormhole_space[wh_class.value];
        return class_data[form.category as keyof typeof class_data];
    }

    if (security.value >= 0.5) {
        const hs_data = signature_tree.known_space.hs;
        return hs_data[form.category as keyof typeof hs_data];
    }

    if (security.value >= 0.1) {
        const ls_data = signature_tree.known_space.ls;
        return ls_data[form.category as keyof typeof ls_data];
    }

    const ns_data = signature_tree.known_space.ns;
    return ns_data[form.category as keyof typeof ns_data];
});

watch(
    () => form.category,
    () => {
        form.type = null;
    },
);

watch(
    () => signature.signature_id,
    () => {
        form.signature_id = signature.signature_id || '';
    },
);
watch(
    () => signature.category,
    () => {
        form.category = signature.category;
    },
);
watch(
    () => signature.type,
    () => {
        form.type = signature.type;
    },
);

function updateId(value: string[]) {
    const value_string = value.join('');
    form.signature_id = `${value_string.slice(0, 3)}-${value_string.slice(3, 6)}`.toUpperCase();
}

function handleSubmit() {
    form.put(route('signatures.update', signature.id), {
        preserveScroll: true,
        preserveState: true,
        only: ['selected_map_solarsystem'],
        onSuccess: () => {
            form.reset();
        },
    });
}

function handleDelete() {
    form.delete(route('signatures.destroy', signature.id), {
        preserveScroll: true,
        preserveState: true,
        only: ['selected_map_solarsystem'],
    });
}
</script>

<template>
    <TableRow
        class="data-[deleted=true]:bg-red-500/10 data-[new=true]:bg-green-500/10 data-[updated=true]:bg-amber-500/10"
        :data-deleted="deleted"
        :data-new="$props.new"
        :data-updated="updated"
    >
        <TableCell>
            <div class="flex items-center gap-2">
                <div
                    class="inline-block size-2 rounded-full data-[scanned=false]:bg-red-500 data-[scanned=true]:bg-green-500"
                    :data-scanned="Boolean(signature.signature_id && signature.type)"
                />
                <PinInput :model-value="form.signature_id?.replace('-', '').split('')" @update:modelValue="updateId">
                    <PinInputGroup>
                        <PinInputSlot :index="0" />
                        <PinInputSlot :index="1" />
                        <PinInputSlot :index="2" />
                        <PinInputSeparator />
                        <PinInputSlot :index="3" />
                        <PinInputSlot :index="4" />
                        <PinInputSlot :index="5" />
                    </PinInputGroup>
                </PinInput>
            </div>
        </TableCell>
        <TableCell>
            <Select v-model:model-value="form.category">
                <SelectTrigger class="w-full">
                    <SelectValue placeholder="Select category" />
                </SelectTrigger>
                <SelectContent>
                    <SelectItem v-for="option in options" :key="option" :value="option">
                        {{ option }}
                    </SelectItem>
                </SelectContent>
            </Select>
        </TableCell>
        <TableCell class="w-full">
            <Select v-model:model-value="form.type">
                <SelectTrigger class="w-full">
                    <SelectValue placeholder="Select type" />
                </SelectTrigger>
                <SelectContent>
                    <SelectItem v-for="option in signature_options" :key="option" :value="option">
                        {{ option }}
                    </SelectItem>
                </SelectContent>
            </Select>
        </TableCell>
        <TableCell>
            {{ modified_at }}
        </TableCell>
        <TableCell>
            <div class="flex gap-2">
                <Tooltip>
                    <TooltipTrigger as-child>
                        <Button variant="secondary" @click="handleDelete">
                            <TrashIcon />
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent> Delete Signature</TooltipContent>
                </Tooltip>
                <Tooltip v-if="form.isDirty">
                    <TooltipTrigger as-child>
                        <Button variant="secondary" @click="handleSubmit">
                            <CheckIcon />
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent> Save Signature</TooltipContent>
                </Tooltip>
            </div>
        </TableCell>
    </TableRow>
</template>

<style scoped></style>
