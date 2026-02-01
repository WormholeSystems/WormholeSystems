<script setup lang="ts">
import DiscordIcon from '@/components/icons/DiscordIcon.vue';
import Logo from '@/components/icons/Logo.vue';
import { Button } from '@/components/ui/button';
import AppLayout from '@/layouts/AppLayout.vue';
import SeoHead from '@/layouts/SeoHead.vue';
import { UTCDate } from '@date-fns/utc';
import { usePage } from '@inertiajs/vue3';
import { format } from 'date-fns';
import { ExternalLink, Github, Globe, Heart, Map, Radio, Route, ScanSearch, Shield, Users, Zap } from 'lucide-vue-next';

const page = usePage();
const currentYear = format(new UTCDate(), 'yyyy');

const features = [
    { icon: Map, title: 'Interactive Maps', description: 'Real-time wormhole mapping with intuitive drag-and-drop controls.' },
    { icon: Users, title: 'Collaborative Mapping', description: 'Seamless real-time collaboration with your corpmates and friends.' },
    { icon: Route, title: 'Smart Routing', description: 'Intelligent pathfinding through complex wormhole chains with mass tracking.' },
    { icon: ScanSearch, title: 'Signature Tracking', description: 'Full signature management with paste-from-clipboard support.' },
    { icon: Shield, title: 'Access Control', description: 'Granular permissions for characters, corporations, and alliances.' },
    { icon: Radio, title: 'Activity Monitoring', description: 'Killmail tracking and intel for your wormhole chain.' },
    { icon: Zap, title: 'EVE Client Integration', description: 'Set waypoints directly from the map to your in-game autopilot.' },
];

const links = [
    { title: 'GitHub', href: 'https://github.com/wormholesystems', icon: Github, description: 'Source code and issue tracker' },
    { title: 'EVE Online', href: 'https://www.eveonline.com', icon: Globe, description: 'The game we all love' },
    { title: 'EVE Developer Docs', href: 'https://developers.eveonline.com', icon: Globe, description: 'ESI API documentation' },
    { title: 'EVE Scout', href: 'https://www.eve-scout.com', icon: Globe, description: 'Thera & Turnur connections' },
];
</script>

<template>
    <AppLayout>
        <SeoHead
            title="About — WormholeSystems"
            description="About WormholeSystems, the modern wormhole mapping platform for EVE Online."
            keywords="about, wormhole, mapping, eve online"
        />

        <div class="mx-auto max-w-4xl space-y-12 px-6 py-12">
            <!-- Hero -->
            <div class="space-y-4">
                <div class="flex items-center gap-3">
                    <Logo class="size-8 fill-current text-foreground" />
                    <h1 class="text-3xl font-bold tracking-tight">WormholeSystems</h1>
                </div>
                <p class="max-w-2xl text-lg text-muted-foreground">
                    The modern mapping platform for the wormhole community. Built for capsuleers by capsuleers.
                </p>
            </div>

            <!-- Discord -->
            <section class="rounded-lg border border-border/50 bg-card p-6">
                <div class="flex items-start gap-4">
                    <div class="rounded-lg bg-muted p-2.5">
                        <DiscordIcon class="size-5 text-foreground" />
                    </div>
                    <div class="flex-1 space-y-3">
                        <h2 class="text-lg font-semibold">Join the Community</h2>
                        <p class="text-sm text-muted-foreground">
                            Get help, share feedback, report bugs, and connect with other capsuleers on our Discord server. This is where development
                            discussions happen and new features are announced.
                        </p>
                        <Button as-child size="sm">
                            <a :href="page.props.discord.invite" target="_blank" rel="noopener noreferrer" class="inline-flex items-center gap-2">
                                <DiscordIcon class="size-4" />
                                Join Discord
                                <ExternalLink class="size-3" />
                            </a>
                        </Button>
                    </div>
                </div>
            </section>

            <!-- Features -->
            <section class="space-y-4">
                <h2 class="text-lg font-semibold">Features</h2>
                <div class="grid gap-3 sm:grid-cols-2">
                    <div
                        v-for="feature in features"
                        :key="feature.title"
                        class="flex items-start gap-3 rounded-lg border border-border/50 bg-card p-4"
                    >
                        <component :is="feature.icon" class="mt-0.5 size-4 shrink-0 text-muted-foreground" />
                        <div>
                            <h3 class="text-sm font-medium">{{ feature.title }}</h3>
                            <p class="mt-1 text-xs text-muted-foreground">{{ feature.description }}</p>
                        </div>
                    </div>
                </div>
            </section>

            <!-- Support -->
            <section class="rounded-lg border border-border/50 bg-card p-6">
                <div class="flex items-start gap-4">
                    <div class="rounded-lg bg-muted p-2.5">
                        <Heart class="size-5 text-foreground" />
                    </div>
                    <div class="flex-1 space-y-3">
                        <h2 class="text-lg font-semibold">Support the Project</h2>
                        <p class="text-sm text-muted-foreground">
                            WormholeSystems is free and open source. If you find it useful, consider supporting development to help cover server costs
                            and fund new features.
                        </p>
                        <Button as-child size="sm" variant="outline">
                            <a
                                href="https://github.com/sponsors/wormholesystems"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="inline-flex items-center gap-2"
                            >
                                <Github class="size-4" />
                                GitHub Sponsors
                                <ExternalLink class="size-3" />
                            </a>
                        </Button>
                    </div>
                </div>
            </section>

            <!-- External Links -->
            <section class="space-y-4">
                <h2 class="text-lg font-semibold">Links</h2>
                <div class="grid gap-3 sm:grid-cols-2">
                    <a
                        v-for="link in links"
                        :key="link.title"
                        :href="link.href"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="group flex items-start gap-3 rounded-lg border border-border/50 bg-card p-4 transition-colors hover:border-border"
                    >
                        <component :is="link.icon" class="mt-0.5 size-4 shrink-0 text-muted-foreground" />
                        <div>
                            <h3 class="text-sm font-medium group-hover:text-foreground">{{ link.title }}</h3>
                            <p class="mt-1 text-xs text-muted-foreground">{{ link.description }}</p>
                        </div>
                    </a>
                </div>
            </section>

            <!-- Credits -->
            <footer class="border-t border-border/50 pt-8 pb-4">
                <div class="space-y-2 text-xs text-muted-foreground/60">
                    <p>&copy; {{ currentYear }} WormholeSystems. All rights reserved.</p>
                    <p>EVE Online and the EVE logo are the registered trademarks of CCP hf. All rights are reserved worldwide.</p>
                    <p>All EVE-related materials are property of CCP hf. WormholeSystems is not affiliated with or endorsed by CCP hf.</p>
                </div>
            </footer>
        </div>
    </AppLayout>
</template>
