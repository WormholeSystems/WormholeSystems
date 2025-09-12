import { AppPageProps } from '@/types';
import { router, usePage } from '@inertiajs/vue3';
import { watch } from 'vue';
import { Action, toast } from 'vue-sonner';

export type TNotification = {
    id?: string;
    type: string;
    title: string;
    message: string;
    action?: {
        title: string;
        url: string;
        external: boolean;
    };
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
                    action: getToastAction(notification.action),
                });
            }
        },
        { deep: true, immediate: true },
    );
}

function getToastAction(action: TNotification['action']): Action | undefined {
    if (!action) return undefined;

    if (action.external) {
        return {
            label: action.title,
            onClick: () => {
                window.open(action.url, '_blank', 'noopener noreferrer');
            },
        };
    }

    return {
        label: action.title,
        onClick: () => {
            router.visit(action.url);
        },
    };
}
