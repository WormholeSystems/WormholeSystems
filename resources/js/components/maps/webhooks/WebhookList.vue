<script setup lang="ts">
import PlusIcon from '@/components/icons/PlusIcon.vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { TMapWebhook } from '@/types/models';
import { Pencil, Trash2 } from 'lucide-vue-next';

defineProps<{
    title: string;
    emptyText: string;
    webhooks: TMapWebhook[];
    canManage: boolean;
}>();

const emit = defineEmits<{
    add: [];
    edit: [webhook: TMapWebhook];
    delete: [id: number];
}>();
</script>

<template>
    <Card>
        <CardHeader class="flex flex-row items-start justify-between gap-4">
            <div class="space-y-1.5">
                <CardTitle class="text-xl font-semibold">{{ title }}</CardTitle>
                <CardDescription><slot name="description" /></CardDescription>
            </div>
            <Button v-if="canManage" variant="outline" size="sm" @click="emit('add')">
                <PlusIcon class="mr-2 h-4 w-4" />
                Add alert
            </Button>
        </CardHeader>
        <CardContent class="space-y-4">
            <p v-if="webhooks.length === 0" class="text-sm text-muted-foreground">{{ emptyText }}</p>

            <ul v-else class="divide-y divide-border rounded-lg border">
                <li v-for="webhook in webhooks" :key="webhook.id" class="flex items-center gap-3 px-3 py-2">
                    <slot name="icon" :webhook="webhook" />
                    <div class="min-w-0">
                        <div class="flex items-center gap-2">
                            <span class="font-medium">{{ webhook.name }}</span>
                            <Badge v-if="!webhook.is_active" variant="outline">Inactive</Badge>
                        </div>
                        <div class="text-sm text-muted-foreground">
                            <slot name="detail" :webhook="webhook" />
                            <template v-if="webhook.last_fired_at"> · last fired {{ new Date(webhook.last_fired_at).toLocaleString() }}</template>
                        </div>
                    </div>
                    <div v-if="canManage" class="ml-auto flex items-center gap-1">
                        <Button variant="ghost" size="icon" class="text-muted-foreground" @click="emit('edit', webhook)">
                            <Pencil class="h-4 w-4" />
                        </Button>
                        <Button variant="ghost" size="icon" class="text-muted-foreground hover:text-destructive" @click="emit('delete', webhook.id)">
                            <Trash2 class="h-4 w-4" />
                        </Button>
                    </div>
                </li>
            </ul>
        </CardContent>
    </Card>
</template>
