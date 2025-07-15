<script setup lang="ts">
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { TMapUserSetting } from '@/types/models';
import { Link } from '@inertiajs/vue3';

const { map_user_setting } = defineProps<{
    map_user_setting: TMapUserSetting;
}>();
</script>

<template>
    <Dialog :open="!map_user_setting.tracking_allowed">
        <DialogContent>
            <DialogHeader>
                <DialogTitle> Map tracking consent</DialogTitle>
                <DialogDescription>
                    In order to use this map, you must allow tracking of your location. This is necessary to provide accurate map features and
                    updates.
                </DialogDescription>
            </DialogHeader>
            <DialogFooter>
                <Button variant="secondary" as-child>
                    <Link :href="route('home')"> Deny and return to home</Link>
                </Button>
                <Button as-child>
                    <Link
                        :href="route('map-user-settings.update', map_user_setting.id)"
                        :data="{
                            tracking_allowed: true,
                        }"
                        method="put"
                    >
                        Accept and continue
                    </Link>
                </Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>
</template>

<style scoped></style>
