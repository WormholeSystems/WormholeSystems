<script setup lang="ts">
import LockIcon from '@/components/icons/LockIcon.vue';
import Logo from '@/components/icons/Logo.vue';
import { Button } from '@/components/ui/button';
import MapPanelHeader from '@/components/ui/map-panel/MapPanelHeader.vue';
import Notifications from '@/components/user/Notifications.vue';
import SeoHead from '@/layouts/SeoHead.vue';
import Eve from '@/routes/eve';
import { UTCDate } from '@date-fns/utc';
import { Link, usePage } from '@inertiajs/vue3';
import { format } from 'date-fns';
import { computed } from 'vue';

const page = usePage();
const error = computed(() => page.props.errors?.eve);

const currentYear = format(new UTCDate(), 'yyyy');
</script>

<template>
    <div class="relative flex min-h-screen flex-col items-center justify-center overflow-hidden bg-background p-6">
        <SeoHead
            title="Login"
            description="Sign in to WormholeSystems with your EVE Online character to access advanced wormhole mapping tools."
            url="https://wormhole.systems/login"
        />
        <div class="login-backdrop" aria-hidden="true" />

        <div class="relative w-full max-w-md">
            <!-- Logo and brand -->
            <div class="mb-8 flex flex-col items-center gap-4">
                <Link href="/" class="flex items-center justify-center">
                    <Logo class="text-4xl text-foreground" />
                </Link>
                <div class="text-center">
                    <h1 class="font-display text-2xl font-bold tracking-tight text-foreground">WormholeSystems</h1>
                    <p class="mt-1.5 font-mono text-[10px] tracking-wider text-muted-foreground uppercase">EVE Online wormhole mapping</p>
                </div>
            </div>

            <!-- Login card, styled like a panel on the map -->
            <div class="auth-card">
                <MapPanelHeader>
                    wormhole.systems · sign in
                    <template #actions>
                        <span class="flex items-center gap-1.5">
                            <span class="size-2 rounded-full bg-hostile/70" />
                            <span class="size-2 rounded-full bg-active/70" />
                            <span class="size-2 rounded-full bg-empty/70" />
                        </span>
                    </template>
                </MapPanelHeader>
                <div class="flex flex-col gap-3 p-6 sm:p-8">
                    <p class="mb-3 text-center text-sm leading-6 text-muted-foreground">
                        Sign in or create your account with EVE Online to start mapping wormhole space.
                    </p>

                    <div
                        v-if="error"
                        class="rounded border border-red-500/50 bg-red-500/10 px-3 py-2.5 text-center text-sm text-red-600 dark:text-red-400"
                    >
                        {{ error }}
                    </div>

                    <Button asChild size="lg">
                        <a :href="Eve.show().url" class="flex items-center justify-center gap-3">
                            <LockIcon />
                            Sign in with EVE Online
                        </a>
                    </Button>
                    <Button asChild size="lg" variant="outline">
                        <a :href="Eve.show({ query: { without_scopes: true } }).url" class="flex items-center justify-center gap-3">
                            <LockIcon />
                            Sign in without scopes
                        </a>
                    </Button>

                    <p class="mt-3 text-center font-mono text-[10px] tracking-wider text-muted-foreground/70 uppercase">
                        ESI-secure · Official EVE Online SSO · Free to use
                    </p>
                </div>
            </div>

            <!-- Footer -->
            <div class="mt-8 text-center font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">
                <p>© {{ currentYear }} WormholeSystems</p>
                <p class="mt-1">EVE Online and the EVE logo are trademarks of CCP hf.</p>
            </div>
        </div>
    </div>
    <Notifications />
</template>

<style scoped>
/* Same elevated surface as the landing page's section cards. */
.auth-card {
    position: relative;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border-radius: 0.25rem;
    border: 1px solid var(--color-neutral-300);
    background: var(--color-white);
    box-shadow: 0 20px 45px -20px rgb(0 0 0 / 0.35);
}

.dark .auth-card {
    border-color: var(--color-neutral-700);
    background: var(--color-neutral-900);
    box-shadow: 0 20px 45px -20px rgb(0 0 0 / 0.8);
}

/* Dotted map canvas with a soft orange glow, fading out toward the edges so
   the card sits on a patch of "map" in the middle of the page. */
.login-backdrop {
    position: absolute;
    inset: 0;
    pointer-events: none;
    background-image:
        radial-gradient(40rem 28rem at 50% 40%, color-mix(in oklab, var(--color-orange-400) 7%, transparent), transparent 70%),
        radial-gradient(circle, var(--grid) 1px, transparent 1px);
    background-size:
        auto,
        28px 28px;
    -webkit-mask-image: radial-gradient(52rem 40rem at 50% 45%, #000 35%, transparent 78%);
    mask-image: radial-gradient(52rem 40rem at 50% 45%, #000 35%, transparent 78%);
}
</style>
