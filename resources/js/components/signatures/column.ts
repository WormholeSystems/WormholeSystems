import SortableHeader from '@/components/signatures/SortableHeader.vue';
import { createColumnHelper } from '@tanstack/vue-table';
import { formatDistanceToNowStrict } from 'date-fns';
import { h } from 'vue';

export type TRawSignature = {
    signature_id: string;
    type: string;
    category: string | null;
    name: string | null;
    status: 'new' | 'missing' | 'modified' | 'unchanged' | null;
    created_at?: string;
};

const helpers = createColumnHelper<TRawSignature>();
const columns = [
    helpers.accessor('signature_id', {
        header: ({ column }) =>
            h(
                SortableHeader,
                {
                    direction: column.getIsSorted() ? (column.getIsSorted() === 'asc' ? 'asc' : 'desc') : undefined,
                    onClick: column.getToggleSortingHandler(),
                },
                () => 'ID',
            ),
        cell: ({ row }) =>
            h(
                'span',
                {
                    class: 'text-xs data-[scanned=true]:text-green-500 data-[scanned=false]:text-red-500',
                    'data-scanned': row.original.name !== null,
                },
                row.getValue('signature_id'),
            ),
    }),
    helpers.accessor('category', {
        header: ({ column }) =>
            h(
                SortableHeader,
                {
                    direction: column.getIsSorted() ? (column.getIsSorted() === 'asc' ? 'asc' : 'desc') : undefined,
                    onClick: column.getToggleSortingHandler(),
                },
                () => 'Category',
            ),
        cell: ({ row }) =>
            h(
                'span',
                {
                    class: 'text-xs text-muted-foreground block',
                },
                row.getValue('category') || '-',
            ),
    }),
    helpers.accessor('name', {
        header: ({ column }) =>
            h(
                SortableHeader,
                {
                    direction: column.getIsSorted() ? (column.getIsSorted() === 'asc' ? 'asc' : 'desc') : undefined,
                    onClick: column.getToggleSortingHandler(),
                },
                () => 'Name',
            ),
        cell: ({ row }) =>
            h(
                'span',
                {
                    class: 'text-xs block',
                },
                row.getValue('name') || '-',
            ),
    }),
    helpers.accessor('status', {
        header: ({ column }) =>
            h(
                SortableHeader,
                {
                    direction: column.getIsSorted() ? (column.getIsSorted() === 'asc' ? 'asc' : 'desc') : undefined,
                    onClick: column.getToggleSortingHandler(),
                },
                () => 'Status',
            ),
        cell: ({ row }) => {
            let status: string = row.original.status ?? '';

            const created_at = row.original.created_at;
            if (!status && created_at) {
                const date = new Date(created_at);
                status = formatDistanceToNowStrict(date, { addSuffix: true });
            }

            return h(
                'span',
                {
                    class: `text-center text-xs uppercase block data-[status=new]:text-green-500 data-[status=missing]:text-red-500 data-[status=modified]:text-yellow-500 data-[status=unchanged]:text-gray-500 text-neutral-700`,
                    'data-status': row.getValue('status'),
                },
                status || '-',
            );
        },
    }),
];

export default columns;
