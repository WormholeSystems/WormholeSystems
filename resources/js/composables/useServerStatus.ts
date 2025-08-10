import { useOnClient } from '@/composables/useOnClient';
import { getServerStatusChannelName } from '@/const/channels';
import { ServerStatusUpdatedEvent } from '@/const/events';
import { TServerStatus } from '@/types/models';
import { usePage } from '@inertiajs/vue3';
import { useEchoPublic } from '@laravel/echo-vue';
import { ref } from 'vue';

export function useServerStatus() {
    const page = usePage();

    const server_status = ref(page.props.server_status);

    useOnClient(() => {
        useEchoPublic<{
            server_status: TServerStatus;
        }>(getServerStatusChannelName(), ServerStatusUpdatedEvent, (event) => {
            server_status.value = event.server_status;
        });
    });

    return server_status;
}
