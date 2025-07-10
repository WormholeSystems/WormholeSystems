import { AppPageProps } from '@/types';
import { usePage } from '@inertiajs/vue3';
import { watch } from 'vue';
import { toast } from 'vue-sonner';

export type TNotification = {
    id?: string;
    type: string;
    title: string;
    message: string;
};

export function useNotifications() {
    const page = usePage<
        AppPageProps<{
            notification?: TNotification;
        }>
    >();

    watch(
        () => page.props.notification,
        (notification) => {
            if (notification) {
                toast(notification.title, {
                    description: notification.message,
                });
            }
        },
        { deep: true, immediate: true },
    );
}
