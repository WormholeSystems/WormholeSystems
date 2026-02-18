<script setup lang="ts">
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { UTCDate } from '@date-fns/utc';
import { addDays, addHours, addMonths, addWeeks, format, formatDistanceToNowStrict, roundToNearestHours } from 'date-fns';
import { Clock, X } from 'lucide-vue-next';
import { computed, ref, watch } from 'vue';

const props = defineProps<{
    expiresAt: string | null;
}>();

const emit = defineEmits<{
    update: [value: string | null];
}>();

const open = ref(false);
const customDatetime = ref('');

watch(open, (isOpen) => {
    if (isOpen) {
        const rounded = roundToNearestHours(addHours(new UTCDate(), 1), { roundingMethod: 'ceil' });
        customDatetime.value = format(rounded, "yyyy-MM-dd'T'HH:mm");
    }
});

const displayLabel = computed(() => {
    if (!props.expiresAt) {
        return 'Never';
    }

    const date = new UTCDate(props.expiresAt);
    if (date <= new UTCDate()) {
        return 'Expired';
    }

    return formatDistanceToNowStrict(date, { addSuffix: true });
});

const isExpired = computed(() => {
    if (!props.expiresAt) {
        return false;
    }
    return new UTCDate(props.expiresAt) <= new UTCDate();
});

function selectPreset(date: UTCDate) {
    emit('update', date.toISOString());
    open.value = false;
}

function applyCustom() {
    if (!customDatetime.value) {
        return;
    }
    const date = new UTCDate(customDatetime.value);
    emit('update', date.toISOString());
    open.value = false;
    customDatetime.value = '';
}

function removeExpiry() {
    emit('update', null);
    open.value = false;
}

const minDatetime = computed(() => format(new UTCDate(), "yyyy-MM-dd'T'HH:mm"));
</script>

<template>
    <Popover v-model:open="open">
        <PopoverTrigger as-child>
            <button class="cursor-pointer">
                <Badge :variant="expiresAt ? 'outline' : 'secondary'" :class="[isExpired && 'border-destructive text-destructive']">
                    <Clock class="size-3" />
                    {{ displayLabel }}
                </Badge>
            </button>
        </PopoverTrigger>
        <PopoverContent align="start" class="w-64 space-y-3">
            <p class="text-sm font-medium">Set expiry</p>
            <div class="grid grid-cols-2 gap-2">
                <Button variant="outline" size="sm" @click="selectPreset(addHours(new UTCDate(), 1))">1 hour</Button>
                <Button variant="outline" size="sm" @click="selectPreset(addHours(new UTCDate(), 6))">6 hours</Button>
                <Button variant="outline" size="sm" @click="selectPreset(addDays(new UTCDate(), 1))">1 day</Button>
                <Button variant="outline" size="sm" @click="selectPreset(addWeeks(new UTCDate(), 1))">1 week</Button>
                <Button variant="outline" size="sm" class="col-span-2" @click="selectPreset(addMonths(new UTCDate(), 1))"> 1 month </Button>
            </div>
            <div class="space-y-2">
                <label class="text-xs text-muted-foreground">Custom</label>
                <input
                    v-model="customDatetime"
                    type="datetime-local"
                    :min="minDatetime"
                    class="h-8 w-full rounded-md border bg-background px-2 text-sm"
                />
                <Button variant="outline" size="sm" class="w-full" :disabled="!customDatetime" @click="applyCustom"> Apply </Button>
            </div>
            <Button v-if="expiresAt" variant="ghost" size="sm" class="w-full text-destructive" @click="removeExpiry">
                <X class="size-3" />
                Remove expiry
            </Button>
        </PopoverContent>
    </Popover>
</template>
