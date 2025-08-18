<script setup lang="ts">
import Audit from '@/components/audits/Audit.vue';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { TAudit } from '@/types/models';

defineProps<{
    audits: TAudit[];
}>();
</script>

<template>
    <Card class="pb-0">
        <CardHeader>
            <CardTitle> Audits</CardTitle>
            <CardDescription> A list of the last changes made to the map solarsystem.</CardDescription>
        </CardHeader>
        <CardContent class="p-1">
            <div class="relative max-h-60 overflow-x-hidden overflow-y-scroll mask-b-from-90% mask-alpha pr-1">
                <div class="rounded-lg border bg-white dark:bg-neutral-900/40">
                    <div class="grid grid-cols-[auto_1fr_auto_auto] gap-x-4">
                        <Audit v-for="audit in audits" :key="audit.id" :audit="audit" />
                        <div v-if="audits.length === 0" class="col-span-4 p-4 text-center text-sm text-muted-foreground">
                            No audits found for this solarsystem.
                        </div>
                    </div>
                </div>
            </div>
        </CardContent>
    </Card>
</template>

<style scoped></style>
