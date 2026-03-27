import { createInertiaApp, router } from '@inertiajs/vue3';
import { configureEcho } from '@laravel/echo-vue';
import '../css/app.css';
import { initializeTheme } from './composables/useAppearance';
import { isPWA } from './composables/useOnClient';
import { initializeRouting } from './composables/useRoutingWorker';
import { preloadSovereigntyData } from './composables/useSovereigntyData';
import { preloadStaticData } from './composables/useStaticData';

configureEcho({
    broadcaster: 'reverb',
});

router.on('finish', () => {
    router.flushAll();
});

void preloadStaticData();
void initializeRouting();
void preloadSovereigntyData();

const appName = import.meta.env.VITE_APP_NAME || 'Laravel';

createInertiaApp({
    title: (title) => {
        if (isPWA()) {
            return 'wormhole.systems';
        }
        return title ? `${title} | ${appName}` : appName;
    },
    progress: {
        color: '#4B5563',
        delay: 1_000,
    },
});

initializeTheme();
