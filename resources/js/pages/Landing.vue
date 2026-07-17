<script setup lang="ts">
import CharactersView from '@/components/characters/CharactersView.vue';
import DiscordIcon from '@/components/icons/DiscordIcon.vue';
import Logo from '@/components/icons/Logo.vue';
import { AllianceLogo, CharacterImage, CorporationLogo } from '@/components/images';
import CountUp from '@/components/landing/CountUp.vue';
import { buildKillmails, buildSignatures, CHARACTERS, MAP_CONNECTIONS, MAP_PILOTS, MAP_SOLARSYSTEMS } from '@/components/landing/fixtures';
import KillmailsView, { type TKillmailViewModel } from '@/components/map-killmails/KillmailsView.vue';
import SignaturesView from '@/components/signatures/SignaturesView.vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import MapPanelHeader from '@/components/ui/map-panel/MapPanelHeader.vue';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { TooltipProvider } from '@/components/ui/tooltip';
import Notifications from '@/components/user/Notifications.vue';
import { useStaticSolarsystems } from '@/composables/useStaticSolarsystems';
import useUser from '@/composables/useUser';
import Appearance from '@/layouts/Appearance.vue';
import SeoHead from '@/layouts/SeoHead.vue';
import MapReadonly from '@/map/components/MapReadonly.vue';
import { documentation, home } from '@/routes';
import Eve from '@/routes/eve';
import { UTCDate } from '@date-fns/utc';
import type { TKillmail } from '@/types/models';
import { Link, usePage, usePoll } from '@inertiajs/vue3';
import { useMediaQuery } from '@vueuse/core';
import { format } from 'date-fns';
import {
    Activity,
    ArrowRight,
    Bell,
    BookOpen,
    Check,
    Container,
    Copy,
    Crosshair,
    Crown,
    Eye,
    Github,
    Laptop,
    LayoutGrid,
    Monitor,
    MoreHorizontal,
    Pencil,
    Plus,
    Route,
    Save,
    Settings,
    ShieldCheck,
    Smartphone,
    Sparkles,
    Tablet,
    Telescope,
    Users,
    X,
} from 'lucide-vue-next';
import { computed, onMounted, ref, type Component } from 'vue';

const { killmails } = defineProps<{
    killmails?: TKillmail[];
}>();

const currentYear = format(new UTCDate(), 'yyyy');
const user = useUser();
const page = usePage();

const isCompact = useMediaQuery('(max-width: 1279px)');

// The killmail and signature panels show relative timestamps. Render them only
// after mount so the server markup and the client's first paint always match.
const mounted = ref(false);
onMounted(() => {
    mounted.value = true;
});

// Built per-render so relative timestamps stay in sync between SSR and client.
const KILLMAILS = buildKillmails();

// The killmail feed is real: the latest J-space kills from the database,
// refreshed while the page is open. The demo fixtures only step in when the
// database has none (e.g. a fresh self-hosted instance).
const thirty_seconds_in_ms = 30_000;
usePoll(thirty_seconds_in_ms, { only: ['killmails'] });

const { resolveSolarsystem } = useStaticSolarsystems();

const killmailItems = computed<TKillmailViewModel[]>(() => {
    if (!killmails?.length) {
        return KILLMAILS;
    }
    return killmails.map((killmail) => ({
        killmail,
        solarsystem: resolveSolarsystem(killmail.solarsystem_id),
        alias: null,
    }));
});
const SIGNATURES = buildSignatures();

// Self-hosting: the interactive setup wizard from the containers repo.
const installCommand = "curl --proto '=https' --tlsv1.2 -sSf https://install.wormhole.systems | sh";
const commandCopied = ref(false);
let copyResetTimer: ReturnType<typeof setTimeout> | null = null;

async function copyInstallCommand() {
    await navigator.clipboard.writeText(installCommand);
    commandCopied.value = true;
    if (copyResetTimer !== null) {
        clearTimeout(copyResetTimer);
    }
    copyResetTimer = setTimeout(() => {
        commandCopied.value = false;
    }, 2000);
}

// Layout editor showcase. Mirrors the real floating toolbar + card grid.
// Four cards are always present (map, system info, signatures, notes); these
// eight more can be hidden via the card library.
const hiddenCardCount = 4;

// Access control showcase. Mirrors the real "entities with access" table and
// the Viewer / Member / Manager / Owner permission levels.
type AccessPermission = 'viewer' | 'member' | 'manager' | 'owner';

const permissionMeta: Record<AccessPermission, { label: string; icon: Component; class: string }> = {
    viewer: { label: 'Viewer', icon: Eye, class: 'text-blue-500' },
    member: { label: 'Member', icon: Pencil, class: 'text-green-500' },
    manager: { label: 'Manager', icon: Settings, class: 'text-purple-500' },
    owner: { label: 'Owner', icon: Crown, class: 'text-amber-500' },
};

const accessEntities: Array<{
    id: number;
    name: string;
    type: 'character' | 'corporation' | 'alliance';
    permission: AccessPermission;
    expires: string | null;
}> = [
    { id: 2112000001, name: 'Karima Solette', type: 'character', permission: 'owner', expires: null },
    { id: 99000001, name: 'Hole Control', type: 'alliance', permission: 'manager', expires: null },
    { id: 98000001, name: 'Wandering Phoenix', type: 'corporation', permission: 'member', expires: null },
    { id: 2112000002, name: 'Tovan Khev', type: 'character', permission: 'viewer', expires: 'in 7 days' },
];

const seoData = {
    title: 'WormholeSystems - The new wormhole mapping tool',
    description: 'Map and navigate wormhole space with ease using WormholeSystems, the modern mapping platform for capsuleers.',
    keywords: 'wormhole, mapping, eve online, capsuleers, community, navigation',
};

const stats = [
    { k: 'Signatures resolved', to: 2.4, suffix: 'M', decimals: 1 },
    { k: 'Connections mapped', to: 680, suffix: 'k', decimals: 0 },
    { k: 'ISK destroyed, tracked', to: 9.8, suffix: 'T', decimals: 1 },
    { k: 'Capsuleers aboard', to: 14200, suffix: '', decimals: 0 },
];

const secondaryFeatures = [
    {
        icon: Route,
        title: 'Smart routing',
        body: 'Finds the shortest way home through the chain, taking wormhole mass limits and any systems you have chosen to avoid into account.',
    },
    {
        icon: Sparkles,
        title: 'Intelligence',
        body: 'Keeps notes on your systems automatically, so nobody re-scans what your group already figured out.',
    },
    {
        icon: Crosshair,
        title: 'Threat analysis',
        body: 'Flags systems with recent kills, so you know what you might be jumping into.',
    },
    {
        icon: Bell,
        title: 'Discord alerts',
        body: 'Sends proximity and killmail alerts to your Discord, filtered however you like.',
    },
    {
        icon: Telescope,
        title: 'EVE Scout',
        body: 'Pulls in Thera and Turnur connections and keeps them current, so routing can use them too.',
    },
];

const vReveal = {
    mounted(el: HTMLElement, binding: { value?: string }) {
        if (typeof IntersectionObserver === 'undefined' || window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
            return;
        }
        el.classList.add('reveal', `reveal--${binding.value ?? 'up'}`);
        const io = new IntersectionObserver(
            (entries) => {
                for (const entry of entries) {
                    if (entry.isIntersecting) {
                        el.classList.add('reveal-in');
                        io.unobserve(el);
                    }
                }
            },
            { threshold: 0.1, rootMargin: '0px 0px -6% 0px' },
        );
        io.observe(el);
    },
};
</script>

<template>
    <TooltipProvider :delay-duration="300">
        <div class="relative isolate min-h-screen overflow-x-hidden bg-background text-foreground">
            <SeoHead :title="seoData.title" :description="seoData.description" :keywords="seoData.keywords" />

            <!-- Nav: the app's hairline-and-blur chrome. -->
            <nav class="fixed inset-x-0 top-0 z-50 border-b border-border bg-background/85 backdrop-blur-xl">
                <div class="mx-auto flex h-14 max-w-7xl items-center justify-between px-6 sm:px-10">
                    <div class="flex items-center gap-3">
                        <Logo class="h-6 w-6 text-foreground" />
                        <span class="font-display text-base font-bold tracking-tight text-foreground">WormholeSystems</span>
                    </div>
                    <div class="flex items-center gap-4">
                        <Link :href="documentation().url" class="nav-link hidden items-center gap-2 sm:flex">
                            <BookOpen class="h-3.5 w-3.5" />
                            Docs
                        </Link>
                        <a :href="page.props.discord.invite" class="nav-link hidden items-center gap-2 sm:flex">
                            <DiscordIcon class="h-3.5 w-3.5" />
                            Discord
                        </a>
                        <Appearance />
                        <Button v-if="!user" asChild variant="outline" size="sm">
                            <a :href="Eve.show().url" class="text-sm font-medium">Sign in</a>
                        </Button>
                        <Button v-else asChild size="sm" variant="outline">
                            <Link :href="home()" class="text-sm font-medium" prefetch>Go to maps</Link>
                        </Button>
                    </div>
                </div>
            </nav>

            <main class="relative pt-14">
                <!-- Hero: its own solid band, closed off with a hairline. -->
                <section class="relative overflow-hidden border-b border-border bg-muted/20">
                    <div class="hero-backdrop" aria-hidden="true" />
                    <div class="relative mx-auto max-w-7xl px-6 sm:px-10">
                        <div class="grid items-center gap-14 py-24 lg:grid-cols-[0.85fr_1.15fr] lg:py-32">
                            <div class="hero-intro">
                                <div class="section-label">
                                    <span class="size-1.5 animate-pulse rounded-full bg-green-500" />
                                    Live, interactive wormhole maps
                                </div>
                                <h1
                                    class="mt-7 font-display text-5xl leading-[0.98] font-bold tracking-tight text-foreground sm:text-6xl lg:text-7xl"
                                >
                                    Navigate the
                                    <span class="text-orange-400">Unknown</span>
                                </h1>
                                <p class="mt-7 max-w-xl text-lg leading-8 text-muted-foreground">
                                    Map your wormhole chain, track signatures, and watch for hostiles together. Fly solo, with your corp, or a whole
                                    alliance. Everyone shares the same live map.
                                </p>
                                <div class="mt-9 flex flex-wrap items-center gap-3">
                                    <template v-if="!user">
                                        <Button asChild size="lg">
                                            <a :href="Eve.show().url" class="group inline-flex items-center gap-2">
                                                Sign in with EVE
                                                <ArrowRight class="h-4 w-4 transition-transform group-hover:translate-x-0.5" />
                                            </a>
                                        </Button>
                                        <Button asChild size="lg" variant="outline">
                                            <a :href="Eve.show({ query: { without_scopes: true } }).url" class="inline-flex items-center gap-2">
                                                Sign in without scopes
                                            </a>
                                        </Button>
                                    </template>
                                    <Button asChild size="lg" v-else>
                                        <Link :href="home()" class="group inline-flex items-center gap-2" prefetch>
                                            Explore Maps
                                            <ArrowRight class="h-4 w-4 transition-transform group-hover:translate-x-0.5" />
                                        </Link>
                                    </Button>
                                    <Button as-child size="lg" variant="ghost">
                                        <a :href="page.props.discord.invite" class="inline-flex items-center gap-2">
                                            <DiscordIcon class="h-4 w-4" />
                                            Join the Discord
                                        </a>
                                    </Button>
                                </div>
                                <p class="mt-8 font-mono text-[10px] tracking-wider text-muted-foreground/70 uppercase">
                                    ESI-secure · No client install · Free to use
                                </p>
                            </div>

                            <!-- The real map, framed as an elevated card. The map grid
                                 lives inside this card, where it belongs. -->
                            <div class="hero-console">
                                <div class="section-card">
                                    <MapPanelHeader>
                                        home.map · J152820
                                        <template #actions>
                                            <span class="flex items-center gap-1.5">
                                                <span class="size-2 rounded-full bg-hostile/70" />
                                                <span class="size-2 rounded-full bg-active/70" />
                                                <span class="size-2 rounded-full bg-empty/70" />
                                            </span>
                                        </template>
                                    </MapPanelHeader>
                                    <div class="relative h-[420px] w-full overflow-hidden sm:h-[460px] xl:h-[540px]">
                                        <MapReadonly
                                            :solarsystems="MAP_SOLARSYSTEMS"
                                            :connections="MAP_CONNECTIONS"
                                            :pilots="MAP_PILOTS"
                                            :home_solarsystem_id="1"
                                            :scale="isCompact ? 0.62 : 0.85"
                                        />
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </section>

                <!-- The chain: every section is a card, linked down the middle
                     column like systems in a wormhole chain. -->
                <div class="relative">
                    <div class="canvas-backdrop" aria-hidden="true" />
                    <div class="relative mx-auto max-w-6xl px-6 pb-24 sm:px-10">
                        <!-- Stats -->
                        <div class="chain" aria-hidden="true" />
                        <div v-reveal class="section-card">
                            <div class="hairline-grid grid grid-cols-2 gap-px text-center md:grid-cols-4">
                                <div v-for="stat in stats" :key="stat.k" class="surface-cell px-6 py-10">
                                    <div class="font-display text-4xl font-bold tracking-tight text-foreground">
                                        <CountUp :to="stat.to" :suffix="stat.suffix" :decimals="stat.decimals" />
                                    </div>
                                    <div class="mt-2 font-mono text-[10px] tracking-wider text-muted-foreground uppercase">{{ stat.k }}</div>
                                </div>
                            </div>
                        </div>

                        <!-- Open source & self-hosting (surfaced early — it's a core differentiator). -->
                        <div class="chain" aria-hidden="true" />
                        <div v-reveal class="section-card">
                            <MapPanelHeader>
                                <span class="flex items-center gap-2"><Github class="h-3.5 w-3.5" /> 100% open source</span>
                            </MapPanelHeader>
                            <div class="grid divide-y divide-border/50 lg:grid-cols-[1.1fr_1fr] lg:divide-x lg:divide-y-0">
                                <div class="p-8 sm:p-12">
                                    <h2 class="section-title">Self-hosting is one command away</h2>
                                    <p class="section-lead">
                                        Use the hosted version, dig into the code, or run your own private instance. One command launches the
                                        interactive setup wizard — no lock-in, it's all out in the open.
                                    </p>
                                    <div class="cmd group mt-8">
                                        <code class="cmd-text">{{ installCommand }}</code>
                                        <button
                                            type="button"
                                            class="cmd-copy"
                                            :aria-label="commandCopied ? 'Copied' : 'Copy command'"
                                            @click="copyInstallCommand"
                                        >
                                            <Check v-if="commandCopied" class="size-4 text-green-500" />
                                            <Copy v-else class="size-4" />
                                        </button>
                                    </div>
                                    <p class="mt-3 font-mono text-[10px] tracking-wider text-muted-foreground/70 uppercase">
                                        All you need is a Linux server with Docker — the wizard handles the rest
                                    </p>
                                </div>
                                <div class="flex flex-col divide-y divide-border/50">
                                    <a
                                        href="https://github.com/WormholeSystems/WormholeSystems"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="oss-link group"
                                    >
                                        <Github class="mt-0.5 h-5 w-5 shrink-0 text-muted-foreground" />
                                        <span class="min-w-0 flex-1">
                                            <span class="block font-display text-base font-bold text-foreground">Source code</span>
                                            <span class="mt-1 block text-sm leading-6 text-muted-foreground">
                                                Browse the full source, open issues, and contribute on GitHub.
                                            </span>
                                        </span>
                                        <ArrowRight
                                            class="mt-1 h-4 w-4 shrink-0 text-muted-foreground transition-transform group-hover:translate-x-0.5"
                                        />
                                    </a>
                                    <a
                                        href="https://github.com/WormholeSystems/wormholesystems-containers"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="oss-link group"
                                    >
                                        <Container class="mt-0.5 h-5 w-5 shrink-0 text-muted-foreground" />
                                        <span class="min-w-0 flex-1">
                                            <span class="block font-display text-base font-bold text-foreground">Container stack</span>
                                            <span class="mt-1 block text-sm leading-6 text-muted-foreground">
                                                The Docker setup and setup wizard behind the one-liner.
                                            </span>
                                        </span>
                                        <ArrowRight
                                            class="mt-1 h-4 w-4 shrink-0 text-muted-foreground transition-transform group-hover:translate-x-0.5"
                                        />
                                    </a>
                                </div>
                            </div>
                        </div>

                        <!-- 01: Shared mapping -->
                        <div class="chain" aria-hidden="true" />
                        <div v-reveal class="section-card">
                            <MapPanelHeader>
                                <span class="flex items-center gap-2"><Users class="h-3.5 w-3.5" /> 01 · Shared mapping</span>
                            </MapPanelHeader>
                            <div class="grid divide-y divide-border/50 lg:grid-cols-2 lg:divide-x lg:divide-y-0">
                                <div class="p-8 sm:p-12">
                                    <h2 class="section-title">Everyone on the same map, live</h2>
                                    <p class="section-lead">
                                        When anyone moves, scans a connection, or updates a system, every pilot sees it right away. No pasting
                                        bookmarks into chat, no side spreadsheet to keep in sync.
                                    </p>
                                    <ul class="points">
                                        <li><span class="dot" /> See who is online, what they fly, and where they are</li>
                                        <li><span class="dot" /> Each pilot's route home, with the jump count</li>
                                        <li><span class="dot" /> Every change shows up for everyone instantly</li>
                                    </ul>
                                </div>
                                <div class="flex flex-col">
                                    <div class="cell-header">
                                        <span class="size-1.5 animate-pulse rounded-full bg-green-500" />
                                        Pilots · {{ CHARACTERS.length }}
                                    </div>
                                    <div class="flex-1 overflow-x-auto">
                                        <CharactersView :characters="CHARACTERS" />
                                    </div>
                                </div>
                            </div>
                        </div>

                        <!-- 02: Kill activity -->
                        <div class="chain" aria-hidden="true" />
                        <div v-reveal class="section-card">
                            <MapPanelHeader>
                                <span class="flex items-center gap-2"><Activity class="h-3.5 w-3.5" /> 02 · Kill activity</span>
                                <template #actions>
                                    <span class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">via zKillboard</span>
                                </template>
                            </MapPanelHeader>
                            <div class="grid divide-y divide-border/50 lg:grid-cols-2 lg:divide-x lg:divide-y-0">
                                <div class="p-8 sm:p-12">
                                    <h2 class="section-title">See every kill in your chain</h2>
                                    <p class="section-lead">
                                        Killmails from your systems show up on their own, straight from zKillboard, so you always know where the
                                        fighting is and how much got blown up.
                                    </p>
                                </div>
                                <div class="p-8 sm:p-12">
                                    <ul class="points mt-0">
                                        <li><span class="dot" /> Who died, who got the kill, how many were involved, and the ISK lost</li>
                                        <li><span class="dot" /> Filter to wormhole space, known space, or everything</li>
                                        <li><span class="dot" /> Click any kill to jump to that system on the map</li>
                                    </ul>
                                </div>
                            </div>
                            <div class="border-t border-border/50">
                                <div class="cell-header">
                                    <span class="size-1.5 animate-pulse rounded-full bg-green-500" />
                                    Live · Latest J-space kills · {{ killmailItems.length }}
                                </div>
                                <div class="min-h-[13rem] overflow-x-auto py-1">
                                    <KillmailsView v-if="mounted" :items="killmailItems" />
                                </div>
                            </div>
                        </div>

                        <!-- 03: Signatures -->
                        <div class="chain" aria-hidden="true" />
                        <div v-reveal class="section-card">
                            <MapPanelHeader>
                                <span class="flex items-center gap-2"><Crosshair class="h-3.5 w-3.5" /> 03 · Signatures</span>
                            </MapPanelHeader>
                            <div class="grid divide-y divide-border/50 lg:grid-cols-2 lg:divide-x lg:divide-y-0">
                                <div class="flex flex-col lg:order-1">
                                    <div class="cell-header">Signatures · {{ SIGNATURES.length }}</div>
                                    <div class="min-h-[18rem] flex-1 overflow-x-auto">
                                        <SignaturesView v-if="mounted" :signatures="SIGNATURES" :connections="MAP_CONNECTIONS" />
                                    </div>
                                </div>
                                <div class="p-8 sm:p-12 lg:order-2">
                                    <h2 class="section-title">Scanning is just copy and paste</h2>
                                    <p class="section-lead">
                                        Copy your probe scanner results in game, paste them in, and the map sorts it all out. New signatures get
                                        added, the ones that are gone get removed, and wormhole types line up with their connections automatically.
                                    </p>
                                    <div class="paste-hint">
                                        <span class="kbd">Ctrl</span>
                                        <span class="plus">+</span>
                                        <span class="kbd">V</span>
                                        <span class="paste-text">Paste straight from the in-game probe scanner</span>
                                    </div>
                                    <ul class="points">
                                        <li><span class="dot" /> No formatting and no manual entry</li>
                                        <li><span class="dot" /> Old signatures and dead connections are cleaned up for you</li>
                                        <li><span class="dot" /> Mass and lifetime tracked for you, with end-of-life and critical warnings</li>
                                    </ul>
                                </div>
                            </div>
                        </div>

                        <!-- 04: Customisable widget layout -->
                        <div class="chain" aria-hidden="true" />
                        <div v-reveal class="section-card">
                            <MapPanelHeader>
                                <span class="flex items-center gap-2"><LayoutGrid class="h-3.5 w-3.5" /> 04 · Customisable layout</span>
                            </MapPanelHeader>
                            <div class="grid divide-y divide-border/50 lg:grid-cols-2 lg:divide-x lg:divide-y-0">
                                <div class="p-8 sm:p-12">
                                    <h2 class="section-title">Build the map around you</h2>
                                    <p class="section-lead">
                                        The map is a grid of cards you can drag, resize, and hide. Keep the systems you watch front and centre and
                                        switch off the panels you do not use. Layouts are saved per device, with a separate arrangement for each
                                        screen size.
                                    </p>
                                    <ul class="points">
                                        <li><span class="dot" /> Drag and resize any card, from the map to autopilot to killmails</li>
                                        <li><span class="dot" /> Four cards stay put; eight more can be hidden and brought back any time</li>
                                        <li><span class="dot" /> Responsive breakpoints from mobile to wide desktop, each with its own layout</li>
                                    </ul>
                                </div>
                                <div class="flex flex-col">
                                    <div class="cell-header">
                                        <span class="size-1.5 animate-pulse rounded-full bg-empty" />
                                        Editing layout
                                        <span class="ml-auto font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">J152820</span>
                                    </div>
                                    <div class="relative flex-1 p-3 pb-20">
                                        <div class="wg-grid">
                                            <div class="wg-tile wg-map">Map</div>
                                            <div class="wg-tile">Signatures</div>
                                            <div class="wg-tile">Autopilot</div>
                                            <div class="wg-tile">Characters</div>
                                            <div class="wg-tile">Killmails</div>
                                        </div>
                                        <!-- Faithful replica of the real floating layout-editor toolbar -->
                                        <div class="le-toolbar">
                                            <span class="le-btn"><X class="size-4" /></span>
                                            <span class="le-sep" />
                                            <span class="le-seg">
                                                <span class="le-seg-item"><Smartphone class="size-4" /></span>
                                                <span class="le-seg-item"><Tablet class="size-4" /></span>
                                                <span class="le-seg-item"><Laptop class="size-4" /></span>
                                                <span class="le-seg-item is-active"><Monitor class="size-4" /> Large</span>
                                            </span>
                                            <span class="le-btn"><Plus class="size-4" /></span>
                                            <span class="le-sep" />
                                            <span class="le-btn relative">
                                                <LayoutGrid class="size-4" />
                                                <span class="le-badge">{{ hiddenCardCount }}</span>
                                            </span>
                                            <span class="le-sep" />
                                            <span class="le-save"><Save class="size-3.5" /> Save</span>
                                            <span class="le-btn"><MoreHorizontal class="size-4" /></span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <!-- 05: Access control -->
                        <div class="chain" aria-hidden="true" />
                        <div v-reveal class="section-card">
                            <MapPanelHeader>
                                <span class="flex items-center gap-2"><ShieldCheck class="h-3.5 w-3.5" /> 05 · Access control</span>
                            </MapPanelHeader>
                            <div class="grid divide-y divide-border/50 lg:grid-cols-2 lg:divide-x lg:divide-y-0">
                                <div class="flex flex-col lg:order-1">
                                    <div class="cell-header">Access · J152820</div>
                                    <div class="flex-1 px-2 py-1">
                                        <Table>
                                            <TableHeader>
                                                <TableRow class="border-border/50 hover:bg-transparent">
                                                    <TableHead class="w-10" />
                                                    <TableHead>Name</TableHead>
                                                    <TableHead>Type</TableHead>
                                                    <TableHead>Access level</TableHead>
                                                    <TableHead>Expires</TableHead>
                                                </TableRow>
                                            </TableHeader>
                                            <TableBody>
                                                <TableRow
                                                    v-for="entity in accessEntities"
                                                    :key="entity.type + entity.id"
                                                    class="border-border/50 hover:bg-muted/30"
                                                >
                                                    <TableCell>
                                                        <CharacterImage
                                                            v-if="entity.type === 'character'"
                                                            :character_id="entity.id"
                                                            :character_name="entity.name"
                                                            class="size-8 rounded-lg"
                                                        />
                                                        <CorporationLogo
                                                            v-else-if="entity.type === 'corporation'"
                                                            :corporation_id="entity.id"
                                                            :corporation_name="entity.name"
                                                            class="size-8 rounded-lg"
                                                        />
                                                        <AllianceLogo
                                                            v-else
                                                            :alliance_id="entity.id"
                                                            :alliance_name="entity.name"
                                                            class="size-8 rounded-lg"
                                                        />
                                                    </TableCell>
                                                    <TableCell class="font-medium whitespace-nowrap">{{ entity.name }}</TableCell>
                                                    <TableCell>
                                                        <Badge variant="outline" class="capitalize">{{ entity.type }}</Badge>
                                                    </TableCell>
                                                    <TableCell>
                                                        <Badge
                                                            v-if="entity.permission === 'owner'"
                                                            class="bg-amber-500/10 text-amber-500 hover:bg-amber-500/10"
                                                        >
                                                            <Crown class="mr-1 size-3" />
                                                            Owner
                                                        </Badge>
                                                        <span v-else class="access-pill">
                                                            <component
                                                                :is="permissionMeta[entity.permission].icon"
                                                                class="size-4"
                                                                :class="permissionMeta[entity.permission].class"
                                                            />
                                                            {{ permissionMeta[entity.permission].label }}
                                                        </span>
                                                    </TableCell>
                                                    <TableCell class="whitespace-nowrap text-muted-foreground">
                                                        <span v-if="entity.permission === 'owner'" class="text-muted-foreground/40">—</span>
                                                        <span v-else>{{ entity.expires ?? 'Never' }}</span>
                                                    </TableCell>
                                                </TableRow>
                                            </TableBody>
                                        </Table>
                                    </div>
                                </div>
                                <div class="p-8 sm:p-12 lg:order-2">
                                    <h2 class="section-title">Decide exactly who sees what</h2>
                                    <p class="section-lead">
                                        Four levels of access, from view-only to full control. Viewers read the map, Members contribute signatures and
                                        connections, Managers handle access and settings, and the Owner runs the whole thing.
                                    </p>
                                    <ul class="points">
                                        <li><span class="dot" /> Grant access to a character, a corporation, or a whole alliance</li>
                                        <li><span class="dot" /> Viewer, Member, Manager, and Owner roles, each with clear limits</li>
                                        <li><span class="dot" /> Set an optional expiry for temporary or diplomatic access</li>
                                    </ul>
                                </div>
                            </div>
                        </div>

                        <!-- 06: Everything else -->
                        <div class="chain" aria-hidden="true" />
                        <div v-reveal class="section-card">
                            <MapPanelHeader>
                                <span class="flex items-center gap-2"><Sparkles class="h-3.5 w-3.5" /> 06 · Everything else</span>
                            </MapPanelHeader>
                            <div class="p-8 sm:p-12">
                                <h2 class="section-title">Everything else you need to live in a wormhole</h2>
                                <p class="section-lead">The rest of the tools that make day-to-day wormhole life easier.</p>
                            </div>
                            <div class="hairline-grid grid gap-px border-t border-border/50 sm:grid-cols-2 lg:grid-cols-3">
                                <div v-for="feature in secondaryFeatures" :key="feature.title" class="surface-cell group p-7">
                                    <div class="feat-icon">
                                        <component
                                            :is="feature.icon"
                                            class="h-4.5 w-4.5 text-muted-foreground transition-colors group-hover:text-foreground"
                                        />
                                    </div>
                                    <h3 class="mt-5 font-display text-lg font-bold text-foreground">{{ feature.title }}</h3>
                                    <p class="mt-2.5 text-[15px] leading-7 text-muted-foreground">{{ feature.body }}</p>
                                </div>
                                <!-- Filler cell so the hairline grid stays rectangular on wide screens. -->
                                <div class="surface-cell hidden p-7 lg:block" aria-hidden="true" />
                            </div>
                        </div>
                    </div>
                </div>

                <!-- CTA: the end of the chain. -->
                <section class="cta">
                    <div v-reveal class="relative mx-auto max-w-3xl px-6 text-center sm:px-10">
                        <div class="section-label justify-center">
                            <span class="size-1.5 animate-pulse rounded-full bg-green-500" />
                            Drop your first probe
                        </div>
                        <h2 class="cta-title">Ready to map the <span class="text-orange-400">void</span>?</h2>
                        <p class="cta-lead">Set up a map for your corp, your alliance, or just yourself, and start mapping in minutes.</p>
                        <div class="mt-10 flex justify-center">
                            <Button asChild size="lg">
                                <a :href="Eve.show().url" class="group inline-flex items-center gap-2">
                                    Start exploring
                                    <ArrowRight class="h-4 w-4 transition-transform group-hover:translate-x-0.5" />
                                </a>
                            </Button>
                        </div>
                    </div>
                </section>
            </main>

            <footer class="relative border-t border-border bg-background/70 backdrop-blur-sm">
                <div class="mx-auto flex max-w-7xl flex-col items-center justify-between gap-4 px-6 py-12 sm:flex-row sm:px-10">
                    <div class="flex items-center gap-3">
                        <Logo class="h-6 w-6 text-muted-foreground/60" />
                        <span class="font-display text-sm font-bold text-muted-foreground">WormholeSystems</span>
                    </div>
                    <nav class="flex items-center gap-2">
                        <a
                            href="https://github.com/WormholeSystems/WormholeSystems"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="inline-flex items-center gap-2 rounded-md px-2 py-1.5 text-sm text-muted-foreground transition-colors hover:text-foreground"
                        >
                            <Github class="h-4 w-4" />
                            Source
                        </a>
                        <a
                            href="https://github.com/WormholeSystems/wormholesystems-containers"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="inline-flex items-center gap-2 rounded-md px-2 py-1.5 text-sm text-muted-foreground transition-colors hover:text-foreground"
                        >
                            <Container class="h-4 w-4" />
                            Self-host
                        </a>
                    </nav>
                    <p class="text-center text-sm text-muted-foreground/60 sm:text-right">
                        © {{ currentYear }} WormholeSystems. EVE Online and the EVE logo are trademarks of CCP hf.
                    </p>
                </div>
            </footer>
        </div>
    </TooltipProvider>
    <Notifications />
</template>

<style scoped>
.font-display {
    font-family: var(--font-display);
}

/* Section cards wear the map node-card surface: elevated above the page
   background exactly like a system card sits above the map canvas. */
.section-card {
    --surface: var(--color-white);
    --surface-border: var(--color-neutral-300);
    position: relative;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border-radius: 0.25rem;
    border: 1px solid var(--surface-border);
    background: var(--surface);
    box-shadow: 0 20px 45px -20px rgb(0 0 0 / 0.35);
}

.dark .section-card {
    --surface: var(--color-neutral-900);
    --surface-border: var(--color-neutral-700);
    box-shadow: 0 20px 45px -20px rgb(0 0 0 / 0.8);
}

/* Cells that must repeat the card surface inside hairline grids. */
.surface-cell {
    background: var(--surface);
}

/* Hairline separators between surface cells, at the same strength as the
   card's internal dividers. */
.hairline-grid {
    background: color-mix(in oklab, var(--border) 50%, var(--surface));
}

/* Hero backdrop: dotted canvas plus a soft glow behind the map card, masked
   away from the copy column so text contrast stays untouched. */
.hero-backdrop {
    position: absolute;
    inset: 0;
    pointer-events: none;
    background-image:
        radial-gradient(50rem 30rem at 78% 34%, color-mix(in oklab, var(--color-orange-400) 8%, transparent), transparent 70%),
        radial-gradient(circle, var(--grid) 1px, transparent 1px);
    background-size:
        auto,
        28px 28px;
    -webkit-mask-image: linear-gradient(100deg, transparent 30%, #000 62%);
    mask-image: linear-gradient(100deg, transparent 30%, #000 62%);
}

/* Chain canvas backdrop: a dotted map canvas with faint nebula washes. Dots
   instead of grid lines, so nothing fights the card borders, faded out where
   the canvas meets the hero and the CTA. */
.canvas-backdrop {
    position: absolute;
    inset: 0;
    pointer-events: none;
    background-image:
        radial-gradient(60rem 44rem at 82% 4%, color-mix(in oklab, var(--color-orange-400) 5%, transparent), transparent 70%),
        radial-gradient(52rem 40rem at 12% 38%, color-mix(in oklab, var(--color-sky-400) 4%, transparent), transparent 70%),
        radial-gradient(56rem 42rem at 85% 78%, color-mix(in oklab, var(--color-purple-400) 4%, transparent), transparent 70%),
        radial-gradient(circle, var(--grid) 1px, transparent 1px);
    background-size:
        auto,
        auto,
        auto,
        28px 28px;
    -webkit-mask-image: linear-gradient(to bottom, transparent, #000 5rem, #000 calc(100% - 5rem), transparent);
    mask-image: linear-gradient(to bottom, transparent, #000 5rem, #000 calc(100% - 5rem), transparent);
}

/* Chain connector between section cards: the map's neutral connection stroke
   with node endpoints, running down the middle column. */
.chain {
    position: relative;
    margin-inline: auto;
    height: 5rem;
    width: 1.5px;
    background: color-mix(in oklab, var(--foreground) 22%, transparent);
}

@media (min-width: 640px) {
    .chain {
        height: 7rem;
    }
}

.chain::before,
.chain::after {
    content: '';
    position: absolute;
    left: 50%;
    height: 7px;
    width: 7px;
    transform: translateX(-50%);
    border-radius: 9999px;
    border: 1.5px solid color-mix(in oklab, var(--foreground) 35%, transparent);
    background: var(--background);
}

.chain::before {
    top: -3px;
}

.chain::after {
    bottom: -3px;
}

/* Nav links share the panel-header voice. */
.nav-link {
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 10px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted-foreground);
    transition: color 0.2s ease;
}

.nav-link:hover {
    color: var(--foreground);
}

/* Micro-label in the app's panel-header voice: mono, 10px, uppercase, muted. */
.section-label {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 10px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted-foreground);
}

/* Sub-header for showcase cells inside a section card, mirroring MapPanelHeader. */
.cell-header {
    display: flex;
    height: 2.25rem;
    flex-shrink: 0;
    align-items: center;
    gap: 0.5rem;
    border-bottom: 1px solid color-mix(in oklab, var(--border) 50%, transparent);
    background: color-mix(in oklab, var(--muted) 30%, transparent);
    padding-inline: 0.75rem;
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 10px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted-foreground);
}

/* Section copy */
.section-title {
    font-family: var(--font-display);
    font-size: clamp(1.75rem, 1.3rem + 1.8vw, 2.75rem);
    font-weight: 700;
    line-height: 1.08;
    letter-spacing: -0.02em;
    color: var(--foreground);
}

.section-lead {
    margin-top: 1.25rem;
    max-width: 36rem;
    font-size: 1.0625rem;
    line-height: 1.7;
    color: var(--muted-foreground);
}

.points {
    margin-top: 2.25rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
}

.points li {
    display: flex;
    align-items: flex-start;
    gap: 0.875rem;
    font-size: 1rem;
    line-height: 1.5;
    color: color-mix(in oklab, var(--foreground) 82%, transparent);
}

.dot {
    margin-top: 0.55rem;
    height: 0.375rem;
    width: 0.375rem;
    flex-shrink: 0;
    background: var(--color-orange-400);
}

/* One-command install: inset code row with a copy affordance. */
.cmd {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    border-radius: 0.25rem;
    border: 1px solid var(--surface-border);
    background: var(--background);
    padding: 0.35rem 0.35rem 0.35rem 0.9rem;
}

.cmd-text {
    flex: 1;
    overflow-x: auto;
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 0.8rem;
    line-height: 1.9;
    white-space: nowrap;
    color: var(--foreground);
}

.cmd-copy {
    display: flex;
    height: 2rem;
    width: 2rem;
    flex-shrink: 0;
    cursor: pointer;
    align-items: center;
    justify-content: center;
    border-radius: 0.25rem;
    color: var(--muted-foreground);
    transition:
        color 0.2s ease,
        background-color 0.2s ease;
}

.cmd-copy:hover {
    background: color-mix(in oklab, var(--muted) 60%, transparent);
    color: var(--foreground);
}

/* Open-source link rows inside the card. */
.oss-link {
    display: flex;
    flex: 1;
    align-items: center;
    gap: 1rem;
    padding: 1.75rem 2rem;
    transition: background-color 0.2s ease;
}

.oss-link:hover {
    background: color-mix(in oklab, var(--muted) 25%, transparent);
}

/* Paste hint chip */
.paste-hint {
    margin-top: 2rem;
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    border: 1px dashed var(--border);
    background: color-mix(in oklab, var(--muted) 30%, transparent);
    padding: 0.6rem 0.9rem;
}

.paste-hint .kbd {
    border-radius: 0.4rem;
    border: 1px solid var(--border);
    border-bottom-width: 2px;
    background: var(--background);
    padding: 0.1rem 0.45rem;
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 0.75rem;
    font-weight: 700;
    color: var(--foreground);
}

.paste-hint .plus {
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 0.75rem;
    color: var(--muted-foreground);
}

.paste-hint .paste-text {
    margin-left: 0.35rem;
    font-size: 0.85rem;
    color: var(--muted-foreground);
}

.feat-icon {
    display: flex;
    height: 2.5rem;
    width: 2.5rem;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--border);
    background: color-mix(in oklab, var(--muted) 30%, transparent);
}

/* Layout editor showcase: card grid + floating toolbar replica */
.wg-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 0.5rem;
}

.wg-tile {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    min-height: 3.25rem;
    border: 1px dashed var(--border);
    background: color-mix(in oklab, var(--muted) 30%, transparent);
    padding: 0.5rem 0.6rem;
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 0.7rem;
    letter-spacing: 0.04em;
    color: var(--muted-foreground);
}

.wg-tile::after {
    content: '';
    height: 0.5rem;
    width: 0.5rem;
    align-self: flex-end;
    border-right: 2px solid color-mix(in oklab, var(--foreground) 35%, transparent);
    border-bottom: 2px solid color-mix(in oklab, var(--foreground) 35%, transparent);
}

.wg-map {
    grid-column: 1 / -1;
    min-height: 5rem;
    border-style: solid;
    border-color: color-mix(in oklab, var(--foreground) 30%, var(--border));
    background: color-mix(in oklab, var(--muted) 55%, transparent);
    color: var(--foreground);
}

.le-toolbar {
    position: absolute;
    bottom: 0.85rem;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 0.35rem;
    max-width: calc(100% - 1.5rem);
    border-radius: 1rem;
    border: 1px solid color-mix(in oklab, var(--border) 60%, transparent);
    background: color-mix(in oklab, var(--card) 96%, transparent);
    padding: 0.35rem;
    box-shadow: 0 14px 34px -14px rgb(0 0 0 / 0.55);
    backdrop-filter: blur(8px);
}

.le-btn {
    position: relative;
    display: flex;
    height: 1.9rem;
    width: 1.9rem;
    flex-shrink: 0;
    align-items: center;
    justify-content: center;
    border-radius: 0.6rem;
    color: var(--muted-foreground);
}

.le-sep {
    height: 1.4rem;
    width: 1px;
    flex-shrink: 0;
    background: color-mix(in oklab, var(--border) 70%, transparent);
}

.le-seg {
    display: flex;
    align-items: center;
    gap: 0.1rem;
    border-radius: 0.7rem;
    border: 1px solid color-mix(in oklab, var(--border) 60%, transparent);
    background: color-mix(in oklab, var(--muted) 40%, transparent);
    padding: 0.15rem;
}

.le-seg-item {
    display: flex;
    height: 1.6rem;
    align-items: center;
    gap: 0.3rem;
    border-radius: 0.5rem;
    padding: 0 0.4rem;
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 0.7rem;
    color: var(--muted-foreground);
}

.le-seg-item.is-active {
    background: var(--background);
    color: var(--foreground);
    box-shadow: 0 1px 2px rgb(0 0 0 / 0.12);
}

.le-badge {
    position: absolute;
    top: -0.25rem;
    right: -0.25rem;
    display: flex;
    height: 1rem;
    min-width: 1rem;
    align-items: center;
    justify-content: center;
    border-radius: 9999px;
    background: var(--primary);
    padding: 0 0.2rem;
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 10px;
    color: var(--primary-foreground);
}

.le-save {
    display: flex;
    height: 1.9rem;
    flex-shrink: 0;
    align-items: center;
    gap: 0.35rem;
    border-radius: 0.6rem;
    background: var(--primary);
    padding: 0 0.7rem;
    font-size: 0.8rem;
    font-weight: 500;
    color: var(--primary-foreground);
}

/* Access entities table: access-level pill (mirrors the in-app select trigger) */
.access-pill {
    display: inline-flex;
    height: 2rem;
    align-items: center;
    gap: 0.4rem;
    white-space: nowrap;
    border-radius: 0.5rem;
    border: 1px solid color-mix(in oklab, var(--border) 90%, transparent);
    background: var(--background);
    padding: 0 0.6rem;
    font-size: 0.8rem;
    color: var(--foreground);
}

/* CTA: quiet finale. */
.cta {
    position: relative;
    padding-block: 9rem;
}

.cta-title {
    margin-top: 1.5rem;
    font-family: var(--font-display);
    font-size: clamp(2.5rem, 1.7rem + 3.5vw, 4.25rem);
    font-weight: 700;
    line-height: 1.02;
    letter-spacing: -0.02em;
    color: var(--foreground);
}

.cta-lead {
    margin-top: 1.5rem;
    margin-inline: auto;
    max-width: 34rem;
    font-size: 1.125rem;
    line-height: 1.7;
    color: var(--muted-foreground);
}

/* Entrance + scroll-reveal animations */
.hero-intro {
    animation: rise 0.8s cubic-bezier(0.22, 1, 0.36, 1) both;
}

.hero-console {
    animation: rise 0.9s cubic-bezier(0.22, 1, 0.36, 1) 0.15s both;
}

@keyframes rise {
    from {
        opacity: 0;
        transform: translateY(24px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.reveal {
    opacity: 0;
    transition:
        opacity 0.7s ease,
        transform 0.75s cubic-bezier(0.22, 1, 0.36, 1);
}

.reveal--up {
    transform: translateY(26px);
}

.reveal--left {
    transform: translateX(-32px);
}

.reveal--right {
    transform: translateX(32px);
}

.reveal-in {
    opacity: 1;
    transform: none;
}

/* Staggered bullet entrance once a copy block reveals */
.reveal .points li {
    opacity: 0;
    transform: translateY(10px);
}

.reveal-in .points li {
    opacity: 1;
    transform: none;
    transition:
        opacity 0.5s ease,
        transform 0.5s ease;
}

.reveal-in .points li:nth-child(1) {
    transition-delay: 0.18s;
}

.reveal-in .points li:nth-child(2) {
    transition-delay: 0.3s;
}

.reveal-in .points li:nth-child(3) {
    transition-delay: 0.42s;
}

@media (prefers-reduced-motion: reduce) {
    .hero-intro,
    .hero-console {
        animation: none;
    }
}
</style>
