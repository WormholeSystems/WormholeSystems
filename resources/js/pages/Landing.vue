<script setup lang="ts">
import BrainIcon from '@/components/icons/BrainIcon.vue';
import DiscordIcon from '@/components/icons/DiscordIcon.vue';
import SignatureIcon from '@/components/icons/SignatureIcon.vue';
import TelescopeIcon from '@/components/icons/TelescopeIcon.vue';
import Notifications from '@/components/Notifications.vue';
import { Button } from '@/components/ui/button';
import useUser from '@/composables/useUser';
import Appearance from '@/layouts/Appearance.vue';
import { home } from '@/routes';
import Eve from '@/routes/eve';
import { UTCDate } from '@date-fns/utc';
import { Head, Link, usePage } from '@inertiajs/vue3';
import { format } from 'date-fns';
import { ArrowRight, Globe, Route, Shield, Users, Zap } from 'lucide-vue-next';

const currentYear = format(new UTCDate(), 'yyyy');
const user = useUser();

const page = usePage();

const title = 'Get started';
const description = 'The modern mapping platform for the wormhole community. Built for capsuleers by capsuleers.';
const pageUrl = 'https://wormhole.systems';
const combinedKeywords = 'wormhole, mapping, eve online, capsuleers, community, navigation';
const image = {
    url: '/img/og.png',
    width: 1024,
    height: 768,
    type: 'image/png',
};
</script>

<template>
    <div class="min-h-screen bg-background text-foreground">
        <Head>
            <title>{{ title }}</title>
            <meta :content="description" name="description" />
            <meta :content="combinedKeywords" name="keywords" />
            <meta :content="image.url" property="og:image" />
            <meta :content="image.type" property="og:image:type" />
            <meta :content="description" property="og:description" />
            <meta :content="title" property="og:title" />
            <meta :content="pageUrl" property="og:url" />
            <meta :content="image.url" property="twitter:image" />
            <meta :content="description" property="twitter:description" />
            <meta :content="title" property="twitter:title" />
            <meta :content="pageUrl" property="twitter:url" />
            <meta content="summary_large_image" name="twitter:card" />
            <meta v-if="image" :content="image.width.toFixed(0)" property="og:image:width" />
            <meta v-if="image" :content="image.height.toFixed(0)" property="og:image:height" />
            <meta content="wormhole.systems" name="og:site_name" />
            <meta content="#ffffff" name="theme-color" />
            <meta content="wormhole.systems" name="twitter:site" />
            <meta content="en_US" name="og:locale" />
            <meta content="website" name="og:type" />
        </Head>
        <nav class="fixed top-0 z-50 w-full border-b border-border/50 bg-background/80 backdrop-blur-md">
            <div class="mx-auto max-w-7xl px-6 sm:px-8">
                <div class="flex h-16 items-center justify-between">
                    <div class="flex items-center gap-3">
                        <TelescopeIcon class="h-7 w-7 text-foreground" />
                        <span class="text-lg font-semibold text-foreground">WormholeSystems</span>
                    </div>
                    <div class="flex items-center gap-4">
                        <Appearance />
                        <Button v-if="!user" asChild variant="outline" size="sm">
                            <a :href="Eve.show().url" class="text-sm font-medium">Sign in</a>
                        </Button>
                        <Button v-else asChild size="sm" variant="outline">
                            <Link :href="home()" class="text-sm font-medium">Go to maps</Link>
                        </Button>
                    </div>
                </div>
            </div>
        </nav>
        <main class="pt-16">
            <section class="mx-auto max-w-7xl px-6 sm:px-8">
                <div class="py-24 sm:py-32">
                    <div class="mx-auto max-w-4xl text-center">
                        <h1 class="text-5xl font-bold tracking-tight text-foreground sm:text-7xl">
                            Navigate the
                            <span class="bg-gradient-to-r from-foreground to-foreground/70 bg-clip-text text-transparent"> Unknown </span>
                        </h1>
                        <p class="mt-8 text-xl leading-8 text-muted-foreground sm:text-2xl">
                            The modern mapping platform for the wormhole community.
                            <br class="hidden sm:block" />
                            Built for capsuleers by capsuleers.
                        </p>
                        <div class="mt-12 flex items-center justify-center gap-6">
                            <Button asChild size="lg" v-if="!user">
                                <a :href="Eve.show().url" class="inline-flex items-center gap-2">
                                    Get Started
                                    <ArrowRight class="h-4 w-4" />
                                </a>
                            </Button>
                            <Button asChild size="lg" v-else>
                                <Link :href="home()" class="inline-flex items-center gap-2">
                                    Explore Maps
                                    <ArrowRight class="h-4 w-4" />
                                </Link>
                            </Button>
                            <Button as-child size="lg" variant="outline">
                                <a :href="page.props.discord.invite">
                                    <DiscordIcon class="h-4 w-4" />
                                    Join the Discord
                                </a>
                            </Button>
                        </div>
                    </div>
                </div>
            </section>

            <section class="mx-auto max-w-7xl px-6 pb-24 sm:px-8">
                <div class="grid grid-cols-1 gap-6 md:grid-cols-6">
                    <div
                        class="grid grid-rows-[auto_auto_1fr] gap-8 overflow-hidden rounded-2xl border border-border bg-card p-8 md:col-span-4 md:row-span-2"
                    >
                        <div class="flex items-center gap-3">
                            <div class="rounded-lg bg-muted p-2">
                                <Globe class="h-6 w-6 text-foreground" />
                            </div>
                            <h3 class="text-2xl font-bold text-card-foreground">Interactive Maps</h3>
                        </div>
                        <p class="max-w-md text-lg text-muted-foreground">
                            Real-time wormhole mapping with intuitive drag-and-drop controls. Visualize your chains like never before.
                        </p>
                        <div class="overflow-hidden rounded-lg border border-border">
                            <img src="../../../public/img/features/map.png" class="h-full w-full object-cover dark:hidden" alt="Interactive Maps" />
                            <img
                                src="../../../public/img/features/map_dark.png"
                                class="hidden h-full w-full object-cover dark:block"
                                alt="Interactive Maps"
                            />
                        </div>
                    </div>

                    <div
                        class="grid grid-rows-[auto_auto_1fr] gap-8 overflow-hidden rounded-2xl border border-border bg-card p-8 pr-0 pb-0 md:col-span-2"
                    >
                        <div class="flex items-center gap-3">
                            <div class="rounded-lg bg-muted p-2">
                                <Users class="h-5 w-5 text-foreground/80" />
                            </div>
                            <h3 class="text-lg font-bold text-card-foreground">Collaborative Mapping</h3>
                        </div>
                        <p class="text-sm text-muted-foreground">Seamless real-time collaboration with your friends</p>
                        <div class="flex items-center justify-center overflow-hidden rounded-lg">
                            <img
                                src="../../../public/img/features/characters.png"
                                class="h-full w-full object-cover object-left-top dark:hidden"
                                alt="Access Control"
                            />
                            <img
                                src="../../../public/img/features/characters_dark.png"
                                class="hidden h-full w-full object-cover object-left-top dark:block"
                                alt="Access Control"
                            />
                        </div>
                    </div>
                    <div class="grid grid-rows-[auto_auto_1fr] gap-8 overflow-hidden rounded-2xl border border-border bg-card p-8 pb-0 md:col-span-2">
                        <div class="flex items-center gap-3">
                            <div class="rounded-lg bg-muted p-2">
                                <Shield class="h-5 w-5 text-foreground/80" />
                            </div>
                            <h3 class="text-lg font-bold text-card-foreground">Activity Monitoring</h3>
                        </div>
                        <p class="text-sm text-muted-foreground">See what is happening in your chains and systems in real-time.</p>
                        <div class="flex items-center justify-center overflow-hidden rounded-lg">
                            <img src="../../../public/img/features/killmails.png" class="h-full w-auto dark:hidden" alt="Activity Monitoring" />
                            <img
                                src="../../../public/img/features/killmails_dark.png"
                                class="hidden h-full w-auto dark:block"
                                alt="Activity Monitoring"
                            />
                        </div>
                    </div>
                    <div class="grid grid-rows-[auto_auto_1fr] gap-8 overflow-hidden rounded-2xl border border-border bg-card p-8 pb-0 md:col-span-3">
                        <div class="flex items-center gap-3">
                            <div class="rounded-lg bg-muted p-2">
                                <Zap class="h-5 w-5 text-foreground/80" />
                            </div>
                            <h3 class="text-lg font-bold text-card-foreground">Access control</h3>
                        </div>
                        <p class="mb-4 text-sm text-muted-foreground">
                            Unified access control that does not make you want to pull your hair out. Manage permissions with ease.
                        </p>
                        <div class="flex items-center justify-center overflow-hidden rounded-lg">
                            <img src="../../../public/img/features/access.png" class="h-full w-full object-cover dark:hidden" alt="Access Control" />
                            <img
                                src="../../../public/img/features/access_dark.png"
                                class="hidden h-full w-full object-cover dark:block"
                                alt="Access Control"
                            />
                        </div>
                    </div>
                    <div class="grid grid-rows-[auto_auto_1fr] gap-8 overflow-hidden rounded-2xl border border-border bg-card p-8 pb-0 md:col-span-3">
                        <div class="flex items-center gap-3">
                            <div class="rounded-lg bg-muted p-2">
                                <Route class="h-5 w-5 text-foreground/80" />
                            </div>
                            <h3 class="text-lg font-bold text-card-foreground">Smart Routing</h3>
                        </div>
                        <p class="text-sm text-muted-foreground">Intelligent pathfinding through complex wormhole chains with mass calculations.</p>
                        <div class="flex flex-col justify-end overflow-hidden rounded-lg">
                            <img src="../../../public/img/features/autopilot.png" class="object-contain dark:hidden" alt="Smart Routing" />
                            <img src="../../../public/img/features/autopilot_dark.png" class="hidden object-contain dark:block" alt="Smart Routing" />
                        </div>
                    </div>

                    <div class="grid grid-rows-[auto_auto_1fr] gap-8 overflow-hidden rounded-2xl border border-border bg-card p-8 pb-0 md:col-span-3">
                        <div class="flex items-center gap-3">
                            <div class="rounded-lg bg-muted p-2">
                                <SignatureIcon class="h-5 w-5 text-foreground/80" />
                            </div>
                            <h3 class="text-lg font-bold text-card-foreground">Signature Tracking</h3>
                        </div>
                        <p class="text-sm text-muted-foreground">Everything you need to know about wormhole signatures in one place.</p>
                        <div class="flex items-end justify-end">
                            <img src="../../../public/img/features/signatures.png" class="h-auto w-full dark:hidden" alt="Signature Tracking" />
                            <img
                                src="../../../public/img/features/signatures_dark.png"
                                class="hidden h-auto w-full dark:block"
                                alt="Signature Tracking"
                            />
                        </div>
                    </div>
                    <div class="grid grid-rows-[auto_auto_1fr] gap-8 overflow-hidden rounded-2xl border border-border bg-card p-8 pb-0 md:col-span-3">
                        <div class="flex items-center gap-3">
                            <div class="rounded-lg bg-muted p-2">
                                <BrainIcon class="h-5 w-5 text-foreground/80" />
                            </div>
                            <h3 class="text-lg font-bold text-card-foreground">Intelligence</h3>
                        </div>
                        <p class="text-sm text-muted-foreground">
                            Auto-generated notes and insights about your wormhole systems. Never miss a detail again.
                        </p>
                        <div class="flex items-end justify-end">
                            <img src="../../../public/img/features/intel.png" class="h-auto w-full dark:hidden" alt="Intelligence" />
                            <img src="../../../public/img/features/intel_dark.png" class="hidden h-auto w-full dark:block" alt="Intelligence" />
                        </div>
                    </div>
                </div>
            </section>
            <section class="mx-auto max-w-7xl px-6 pb-24 sm:px-8">
                <div class="bg-topo rounded-3xl border border-border px-6 py-24 text-center sm:px-16">
                    <h2 class="text-3xl font-bold tracking-tight text-card-foreground sm:text-4xl">Ready to map the void?</h2>
                    <p class="mx-auto mt-6 max-w-xl text-lg leading-8 text-muted-foreground">
                        Join thousands of capsuleers exploring wormhole space with precision and confidence.
                    </p>
                    <div class="mt-10">
                        <Button asChild size="lg">
                            <a :href="Eve.show().url">
                                Start Exploring
                                <ArrowRight class="h-4 w-4" />
                            </a>
                        </Button>
                    </div>
                </div>
            </section>
        </main>

        <footer class="border-t border-border bg-background">
            <div class="mx-auto max-w-7xl px-6 py-12 sm:px-8">
                <div class="flex flex-col items-center justify-between gap-4 sm:flex-row">
                    <div class="flex items-center gap-3">
                        <TelescopeIcon class="h-6 w-6 text-muted-foreground/60" />
                        <span class="text-sm font-medium text-muted-foreground">WormholeSystems</span>
                    </div>
                    <p class="text-sm text-muted-foreground/60">
                        © {{ currentYear }} WormholeSystems. EVE Online and the EVE logo are trademarks of CCP hf.
                    </p>
                </div>
            </div>
        </footer>
    </div>
    <Notifications />
</template>
<style>
.bg-topo {
    background-image: url('../../../public/patterns/topo_light.svg');
    background-size: cover;
    background-position: center;
}

html.dark .bg-topo {
    background-image: url('../../../public/patterns/topo.svg');
}
</style>
