<script setup lang="ts">
import { TypeImage } from '@/components/images';
import { CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { TShowMapProps } from '@/pages/maps';
import { AppPageProps } from '@/types';
import { UTCDate } from '@date-fns/utc';
import { usePage } from '@inertiajs/vue3';
import { format } from 'date-fns';

const page = usePage<AppPageProps<TShowMapProps>>();
</script>

<template>
    <MapPanel>
        <CardHeader>
            <CardTitle> Ship History</CardTitle>
            <CardDescription> See who has been using this ship and when.</CardDescription>
        </CardHeader>
        <MapPanelContent>
            <Table>
                <TableHeader>
                    <TableRow>
                        <TableHead></TableHead>
                        <TableHead> Name</TableHead>
                        <TableHead> Character</TableHead>
                        <TableHead> First Seen</TableHead>
                        <TableHead> Last Seen</TableHead>
                    </TableRow>
                </TableHeader>
                <TableBody>
                    <TableRow v-for="ship_history in page.props.ship_history || []" :key="ship_history.id">
                        <TableCell>
                            <TypeImage
                                :type_id="ship_history.ship_type_id"
                                class="size-8 shrink-0 rounded-lg"
                                :type_name="ship_history.ship_type?.name || ''"
                            />
                        </TableCell>
                        <TableCell class="truncate">{{ ship_history.name }}</TableCell>
                        <TableCell>{{ ship_history.character?.name }}</TableCell>
                        <TableCell>{{ format(new UTCDate(ship_history.created_at), 'MMM dd, HH:ii') }}</TableCell>
                        <TableCell>{{ format(new UTCDate(ship_history.updated_at), 'MMM dd, HH:ii') }}</TableCell>
                    </TableRow>
                </TableBody>
            </Table>
        </MapPanelContent>
    </MapPanel>
</template>

<style scoped></style>
