<script setup lang="ts">
import CheckIcon from '@/components/icons/CheckIcon.vue';
import TimesIcon from '@/components/icons/TimesIcon.vue';
import TrashIcon from '@/components/icons/TrashIcon.vue';
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import { PinInput, PinInputGroup, PinInputSeparator, PinInputSlot } from '@/components/ui/pin-input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useMapConnections } from '@/composables/map';
import signature_tree from '@/const/signatures';
import { TShowMapProps } from '@/pages/maps';
import { AppPageProps } from '@/types';
import { TSignature } from '@/types/models';
import { useForm, usePage } from '@inertiajs/vue3';
import { useNow } from '@vueuse/core';
import { differenceInHours, differenceInSeconds, format } from 'date-fns';
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
    map_connection_id: signature.map_connection_id,
    type: signature.type,
}));

const page = usePage<AppPageProps<TShowMapProps>>();

const connections = useMapConnections();

const solarsystem_connectins = computed(() => {
    return connections.value
        .filter(
            (connection) =>
                connection.from_map_solarsystem_id === page.props.selected_map_solarsystem?.id ||
                connection.to_map_solarsystem_id === page.props.selected_map_solarsystem?.id,
        )
        .map((connection) => {
            const target = connection.from_map_solarsystem_id !== page.props.selected_map_solarsystem?.id ? connection.source : connection.target;
            return {
                id: connection.id,
                target,
                is_eol: connection.is_eol,
            };
        });
});

const selected_connection = computed(() => {
    return solarsystem_connectins.value.find((connection) => connection.id === form.map_connection_id) || null;
});

const now = useNow();

const wh_class = computed(() => {
    return (page.props.selected_map_solarsystem?.class as keyof typeof signature_tree.wormhole_space) || null;
});

const security = computed(() => {
    return page.props.selected_map_solarsystem?.solarsystem?.security || 0;
});

const created_at = computed(() => {
    return new Date(signature.created_at);
});

const updated_at = computed(() => {
    return signature.updated_at ? new Date(signature.updated_at) : null;
});

const modified_date = computed(() => {
    if (signature.category === 'Wormhole') {
        return created_at.value;
    }

    return updated_at.value || created_at.value;
});

const modified_at = computed(() => {
    const seconds = differenceInSeconds(now.value, modified_date.value);
    if (seconds < 60) {
        return 'now';
    }

    if (seconds < 3600) {
        return `${Math.floor(seconds / 60)}m`;
    }

    if (seconds < 86400) {
        return `${Math.floor(seconds / 3600)}h`;
    }

    return `${Math.floor(seconds / 86400)}d`;
});

const modified_class = computed(() => {
    const hours = differenceInHours(now.value, modified_date.value);

    if (selected_connection.value?.is_eol) {
        return 'eol';
    }
    if (hours < 8) {
        return 'fresh';
    }
    if (hours < 24) {
        return 'old';
    }

    return 'very-old';
});

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

function handleReset() {
    form.reset();
}
</script>

<template>
    <div
        class="col-span-full grid grid-cols-subgrid items-center py-2 *:first:pl-2 *:last:pr-2 data-[deleted=true]:bg-red-500/10 data-[new=true]:bg-green-500/10 data-[updated=true]:bg-amber-500/15"
        :data-deleted="deleted"
        :data-new="$props.new"
        :data-updated="updated"
    >
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
        <Select v-model:model-value="form.type">
            <SelectTrigger class="w-full overflow-hidden data-[wormhole=false]:col-span-2" :data-wormhole="form.category === 'Wormhole'">
                <SelectValue as-child>
                    <span class="truncate">{{ form.type || 'Select type' }}</span>
                </SelectValue>
            </SelectTrigger>
            <SelectContent>
                <SelectItem v-for="option in signature_options" :key="option" :value="option">
                    {{ option }}
                </SelectItem>
            </SelectContent>
        </Select>
        <Select v-model:model-value="form.map_connection_id" v-if="form.category === 'Wormhole'">
            <SelectTrigger class="w-full">
                <SelectValue as-child>
                    <template v-if="selected_connection">
                        <SolarsystemClass
                            :wormhole_class="selected_connection?.target.class"
                            :security="selected_connection?.target.solarsystem?.security"
                        />
                        <span class="mr-auto truncate" v-if="!selected_connection.target.alias">{{ selected_connection?.target.name }}</span>
                        <span class="mr-auto truncate" v-else>
                            <span class="mr-1">{{ selected_connection?.target.alias }}</span>
                            <span class="text-muted-foreground">{{ selected_connection?.target.name }}</span>
                        </span>
                    </template>
                    <template v-else>
                        <span class="truncate">Select connection</span>
                    </template>
                </SelectValue>
            </SelectTrigger>
            <SelectContent>
                <SelectItem v-for="connection in solarsystem_connectins" :key="connection.id" :value="connection.id">
                    <SolarsystemClass :wormhole_class="connection?.target.class" :security="connection?.target.solarsystem?.security" />
                    <span class="mr-auto truncate" v-if="!connection.target.alias">{{ connection?.target.name }}</span>
                    <span class="mr-auto truncate" v-else>
                        <span class="mr-1">{{ connection?.target.alias }}</span>
                        <span class="text-muted-foreground">{{ connection?.target.name }}</span>
                    </span>
                </SelectItem>
            </SelectContent>
        </Select>
        <Tooltip>
            <TooltipTrigger
                :data-modified-class="modified_class"
                class="whitespace-nowrap data-[modified-class=eol]:text-purple-500 data-[modified-class=fresh]:text-green-500 data-[modified-class=old]:text-yellow-500 data-[modified-class=very-old]:text-red-500"
            >
                {{ modified_at }}
            </TooltipTrigger>
            <TooltipContent class="grid grid-cols-[auto_auto] gap-2">
                <span class="font-semibold">Created at</span>
                <p class="">{{ format(created_at, 'MMM dd, HH:ii') }}</p>
                <span class="font-semibold">Last modified at</span>
                <p class="">{{ updated_at ? format(updated_at, 'MMM dd, HH:ii') : 'Never' }}</p>
            </TooltipContent>
        </Tooltip>
        <div class="flex gap-2">
            <template v-if="form.isDirty">
                <Tooltip>
                    <TooltipTrigger as-child>
                        <Button variant="secondary" size="icon" @click="handleReset">
                            <TimesIcon />
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent> Reset Changes</TooltipContent>
                </Tooltip>
                <Tooltip>
                    <TooltipTrigger as-child>
                        <Button variant="secondary" @click="handleSubmit">
                            <CheckIcon />
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent> Save Signature</TooltipContent>
                </Tooltip>
            </template>
            <Tooltip v-else>
                <TooltipTrigger as-child>
                    <Button variant="secondary" @click="handleDelete">
                        <TrashIcon />
                    </Button>
                </TooltipTrigger>
                <TooltipContent> Delete Signature</TooltipContent>
            </Tooltip>
        </div>
    </div>
</template>

<style scoped></style>
