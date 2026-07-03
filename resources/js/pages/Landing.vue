<script setup lang="ts">
import CharactersView from '@/components/characters/CharactersView.vue';
import DiscordIcon from '@/components/icons/DiscordIcon.vue';
import Logo from '@/components/icons/Logo.vue';
import { AllianceLogo, CharacterImage, CorporationLogo } from '@/components/images';
import CountUp from '@/components/landing/CountUp.vue';
import { buildKillmails, buildSignatures, CHARACTERS, MAP_CONNECTIONS, MAP_PILOTS, MAP_SOLARSYSTEMS } from '@/components/landing/fixtures';
import WormholeBackground from '@/components/landing/WormholeBackground.vue';
import KillmailsView from '@/components/map-killmails/KillmailsView.vue';
import SignaturesView from '@/components/signatures/SignaturesView.vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { TooltipProvider } from '@/components/ui/tooltip';
import Notifications from '@/components/user/Notifications.vue';
import useUser from '@/composables/useUser';
import Appearance from '@/layouts/Appearance.vue';
import SeoHead from '@/layouts/SeoHead.vue';
import MapReadonly from '@/map/components/MapReadonly.vue';
import { documentation, home } from '@/routes';
import Eve from '@/routes/eve';
import { UTCDate } from '@date-fns/utc';
import { Link, usePage } from '@inertiajs/vue3';
import { useMediaQuery } from '@vueuse/core';
import { format } from 'date-fns';
import {
    Activity,
    ArrowRight,
    Bell,
    BookOpen,
    Container,
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
    Radar,
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
import { onMounted, ref, type Component } from 'vue';

const currentYear = format(new UTCDate(), 'yyyy');
const user = useUser();
const page = usePage();

const isCompact = useMediaQuery('(max-width: 1024px)');

// The killmail and signature panels show relative timestamps. Render them only
// after mount so the server markup and the client's first paint always match.
const mounted = ref(false);
onMounted(() => {
    mounted.value = true;
});

// Built per-render so relative timestamps stay in sync between SSR and client.
const KILLMAILS = buildKillmails();
const SIGNATURES = buildSignatures();

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
        color: 'text-orange-400',
        title: 'Smart routing',
        body: 'Finds the shortest way home through the chain, taking wormhole mass limits and any systems you have chosen to avoid into account.',
    },
    {
        icon: Sparkles,
        color: 'text-orange-400',
        title: 'Intelligence',
        body: 'Keeps notes on your systems automatically, so nobody re-scans what your group already figured out.',
    },
    {
        icon: Crosshair,
        color: 'text-orange-400',
        title: 'Threat analysis',
        body: 'Flags systems with recent kills, so you know what you might be jumping into.',
    },
    {
        icon: Bell,
        color: 'text-orange-400',
        title: 'Discord alerts',
        body: 'Sends proximity and killmail alerts to your Discord, filtered however you like.',
    },
    {
        icon: Telescope,
        color: 'text-orange-400',
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
        <div class="relative isolate min-h-screen overflow-x-hidden bg-background text-foreground" style="--sec: var(--color-orange-400)">
            <SeoHead :title="seoData.title" :description="seoData.description" :keywords="seoData.keywords" />
            <WormholeBackground class="fixed inset-0 -z-10" />

            <!-- Stronger blur over the central column: content reads against calm,
                 heavily blurred space while the margins keep more detail. -->
            <div class="center-veil" aria-hidden="true" />

            <!-- Structural frame: the content lives in a central column bounded by two
                 rails; the margins outside carry a fine hatch pattern and signal pulses
                 travel down the rails. Purely decorative, wide screens only. -->
            <div class="page-frame" aria-hidden="true">
                <div class="frame-margin frame-margin--left" />
                <div class="frame-margin frame-margin--right" />
                <div class="rail rail--left"><span class="rail-pulse" /></div>
                <div class="rail rail--right"><span class="rail-pulse rail-pulse--alt" /></div>
            </div>

            <!-- Nav: same instrument-panel language as the windows — hairline accent
                 border, mono HUD links, chamfered action. -->
            <nav class="fixed inset-x-0 top-0 z-50 border-b border-orange-400/15 bg-background/70 backdrop-blur-xl">
                <div class="mx-auto flex h-16 max-w-7xl items-center justify-between px-6 sm:px-12 lg:px-24">
                    <div class="flex items-center gap-3">
                        <Logo class="h-7 w-7 text-foreground" />
                        <span class="font-display text-lg font-bold tracking-tight text-foreground">WormholeSystems</span>
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
                        <Button v-if="!user" asChild variant="outline" size="sm" class="hud-btn">
                            <a :href="Eve.show().url" class="text-sm font-medium">Sign in</a>
                        </Button>
                        <Button v-else asChild size="sm" variant="outline" class="hud-btn">
                            <Link :href="home()" class="text-sm font-medium" prefetch>Go to maps</Link>
                        </Button>
                    </div>
                </div>
            </nav>

            <main class="relative pt-16">
                <!-- Hero -->
                <section class="mx-auto max-w-7xl px-6 sm:px-12 lg:px-24">
                    <div class="grid items-center gap-16 py-28 lg:grid-cols-[1.05fr_1fr] lg:py-44">
                        <div class="hero-intro">
                            <div class="chip">
                                <span class="size-1.5 animate-pulse rounded-full bg-orange-400 shadow-[0_0_8px_var(--color-orange-400)]" />
                                Live, interactive wormhole maps
                            </div>
                            <h1
                                class="mt-7 font-display text-6xl leading-[0.95] font-extrabold tracking-tight text-foreground sm:text-7xl lg:text-8xl"
                            >
                                Navigate the
                                <span
                                    class="text-glow gradient-drift bg-gradient-to-br from-amber-300 via-orange-400 to-red-500 bg-clip-text text-transparent"
                                    >Unknown</span
                                >
                            </h1>
                            <p class="mt-8 max-w-xl text-xl leading-8 font-medium text-muted-foreground">
                                Map your wormhole chain, track signatures, and watch for hostiles together. Fly solo, with your corp, or a whole
                                alliance. Everyone shares the same live map.
                            </p>
                            <div class="mt-10 flex flex-wrap items-center gap-4">
                                <template v-if="!user">
                                    <Button asChild size="lg" class="hud-btn">
                                        <a :href="Eve.show().url" class="group inline-flex items-center gap-2">
                                            Sign in with EVE
                                            <ArrowRight class="h-4 w-4 transition-transform group-hover:translate-x-0.5" />
                                        </a>
                                    </Button>
                                    <Button asChild size="lg" variant="outline" class="hud-btn">
                                        <a :href="Eve.show({ query: { without_scopes: true } }).url" class="inline-flex items-center gap-2">
                                            Sign in without scopes
                                        </a>
                                    </Button>
                                </template>
                                <Button asChild size="lg" v-else class="hud-btn">
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
                            <p class="mt-7 font-hud text-xs tracking-wide text-muted-foreground/70">ESI-secure · No client install · Free to use</p>
                        </div>

                        <!-- Command console: the real map -->
                        <div class="hero-console">
                            <div class="window">
                                <div class="window-bar">
                                    <div class="flex items-center gap-2 font-hud text-[11px] tracking-wide text-muted-foreground">
                                        <Radar class="h-3.5 w-3.5 text-orange-400" />
                                        home.map · J152820
                                    </div>
                                    <div class="flex items-center gap-1.5">
                                        <span class="size-2 rounded-full bg-hostile/70" />
                                        <span class="size-2 rounded-full bg-active/70" />
                                        <span class="size-2 rounded-full bg-empty/70" />
                                    </div>
                                </div>
                                <div class="relative h-[380px] w-full overflow-hidden sm:h-[460px]">
                                    <MapReadonly
                                        :solarsystems="MAP_SOLARSYSTEMS"
                                        :connections="MAP_CONNECTIONS"
                                        :pilots="MAP_PILOTS"
                                        :home_solarsystem_id="1"
                                        :scale="isCompact ? 0.58 : 0.8"
                                    />
                                </div>
                            </div>
                            <div class="console-glow" />
                        </div>
                    </div>
                </section>

                <!-- Stat strip -->
                <section class="mx-auto max-w-7xl px-6 pb-10 sm:px-12 lg:px-24">
                    <div v-reveal class="grid grid-cols-2 gap-px overflow-hidden border border-border/60 bg-border/40 text-center md:grid-cols-4">
                        <div v-for="stat in stats" :key="stat.k" class="stat-cell relative bg-card/70 px-6 py-10 backdrop-blur-sm">
                            <div class="stat-num font-display text-4xl font-extrabold tracking-tight text-foreground">
                                <CountUp :to="stat.to" :suffix="stat.suffix" :decimals="stat.decimals" />
                            </div>
                            <div class="mt-2 font-hud text-[11px] tracking-[0.15em] text-muted-foreground uppercase">{{ stat.k }}</div>
                        </div>
                    </div>
                </section>

                <!-- Open source & self-hosting (surfaced early — it's a core differentiator).
                     Deliberately unboxed: a quiet statement with a link list, not a card. -->
                <section class="mx-auto max-w-7xl px-6 py-24 sm:px-12 sm:py-32 lg:px-24">
                    <div v-reveal class="grid items-start gap-14 lg:grid-cols-[1.1fr_1fr] lg:gap-24">
                        <div>
                            <div
                                class="inline-flex items-center gap-2 border-l-2 border-orange-400 pl-3 font-hud text-[11px] tracking-[0.2em] text-orange-300 uppercase"
                            >
                                <Github class="h-3.5 w-3.5" />
                                100% open source
                            </div>
                            <h2 class="mt-7 font-display text-3xl font-bold tracking-tight text-foreground sm:text-4xl">
                                Free, open source, and yours to self-host
                            </h2>
                            <p class="mt-6 max-w-xl text-[15px] leading-7 text-muted-foreground">
                                Use the hosted version, dig into the code, or spin up your own private instance with the ready-made container setup.
                                No lock-in — it's all out in the open.
                            </p>
                        </div>
                        <div class="lg:pt-9">
                            <a
                                href="https://github.com/WormholeSystems/WormholeSystems"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="oss-link group"
                            >
                                <Github class="mt-0.5 h-5 w-5 shrink-0 text-orange-400" />
                                <span class="min-w-0 flex-1">
                                    <span class="block font-display text-base font-bold text-foreground">Source code</span>
                                    <span class="mt-1 block text-sm leading-6 text-muted-foreground">
                                        Browse the full source, open issues, and contribute on GitHub.
                                    </span>
                                </span>
                                <ArrowRight class="mt-1 h-4 w-4 shrink-0 text-muted-foreground transition-transform group-hover:translate-x-0.5" />
                            </a>
                            <a
                                href="https://github.com/WormholeSystems/wormholesystems-containers"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="oss-link group"
                            >
                                <Container class="mt-0.5 h-5 w-5 shrink-0 text-orange-400" />
                                <span class="min-w-0 flex-1">
                                    <span class="block font-display text-base font-bold text-foreground">Self-host</span>
                                    <span class="mt-1 block text-sm leading-6 text-muted-foreground">
                                        A ready-to-run Docker setup for your own private instance.
                                    </span>
                                </span>
                                <ArrowRight class="mt-1 h-4 w-4 shrink-0 text-muted-foreground transition-transform group-hover:translate-x-0.5" />
                            </a>
                        </div>
                    </div>
                </section>

                <!-- Section 01: Shared mapping (copy left / map-style window right) -->
                <section class="feature-section">
                    <div class="mx-auto max-w-7xl px-6 sm:px-12 lg:px-24">
                        <div v-reveal class="marker">
                            <span class="idx">01</span>
                            <span class="tag"><Users class="h-3.5 w-3.5" /> Shared mapping</span>
                            <span class="line" />
                        </div>
                        <div class="grid items-center gap-16 pt-20 lg:grid-cols-2 lg:gap-24">
                            <div v-reveal="'left'">
                                <h2 class="section-title">Everyone on the same map, live</h2>
                                <p class="section-lead">
                                    When anyone moves, scans a connection, or updates a system, every pilot sees it right away. No pasting bookmarks
                                    into chat, no side spreadsheet to keep in sync.
                                </p>
                                <ul class="points">
                                    <li><span class="dot" /> See who is online, what they fly, and where they are</li>
                                    <li><span class="dot" /> Each pilot's route home, with the jump count</li>
                                    <li><span class="dot" /> Every change shows up for everyone instantly</li>
                                </ul>
                            </div>
                            <div v-reveal="'right'" class="window">
                                <div class="window-bar">
                                    <span class="flex items-center gap-2 font-hud text-[11px] tracking-wide text-muted-foreground">
                                        <span class="size-1.5 animate-pulse rounded-full bg-green-500" /> Pilots
                                        <span class="text-amber-400">{{ CHARACTERS.length }}</span>
                                    </span>
                                </div>
                                <div class="overflow-x-auto bg-card/40">
                                    <CharactersView :characters="CHARACTERS" />
                                </div>
                            </div>
                        </div>
                    </div>
                </section>

                <!-- Section 02: Kill activity (full-width feed) -->
                <section class="feature-section feature-section--alt">
                    <div class="mx-auto max-w-7xl px-6 sm:px-12 lg:px-24">
                        <div v-reveal class="marker">
                            <span class="idx">02</span>
                            <span class="tag"><Activity class="h-3.5 w-3.5" /> Kill activity</span>
                            <span class="line" />
                        </div>
                        <div class="grid gap-8 pt-20 lg:grid-cols-2 lg:items-end">
                            <div v-reveal="'left'">
                                <h2 class="section-title">See every kill in your chain</h2>
                                <p class="section-lead">
                                    Killmails from your systems show up on their own, straight from zKillboard, so you always know where the fighting
                                    is and how much got blown up.
                                </p>
                            </div>
                            <div v-reveal="'right'">
                                <ul class="points lg:pb-1">
                                    <li><span class="dot" /> Who died, who got the kill, how many were involved, and the ISK lost</li>
                                    <li><span class="dot" /> Filter to wormhole space, known space, or everything</li>
                                    <li><span class="dot" /> Click any kill to jump to that system on the map</li>
                                </ul>
                            </div>
                        </div>
                        <div v-reveal class="window mt-16">
                            <div class="window-bar">
                                <span class="font-hud text-[11px] tracking-wide text-muted-foreground"
                                    >Killmails <span class="text-amber-400">{{ KILLMAILS.length }}</span></span
                                >
                                <span class="font-hud text-[10px] tracking-wide text-muted-foreground/60">via zKillboard</span>
                            </div>
                            <div class="min-h-[13rem] overflow-x-auto bg-card/40 py-1">
                                <KillmailsView v-if="mounted" :items="KILLMAILS" />
                            </div>
                        </div>
                    </div>
                </section>

                <!-- Section 03: Signatures (window left / copy right + paste hint) -->
                <section class="feature-section">
                    <div class="mx-auto max-w-7xl px-6 sm:px-12 lg:px-24">
                        <div v-reveal class="marker">
                            <span class="idx">03</span>
                            <span class="tag"><Crosshair class="h-3.5 w-3.5" /> Signatures</span>
                            <span class="line" />
                        </div>
                        <div class="grid items-center gap-16 pt-20 lg:grid-cols-2 lg:gap-24">
                            <div v-reveal="'left'" class="window lg:order-1">
                                <div class="window-bar">
                                    <span class="font-hud text-[11px] tracking-wide text-muted-foreground"
                                        >Signatures <span class="text-amber-400">{{ SIGNATURES.length }}</span></span
                                    >
                                </div>
                                <div class="min-h-[18rem] overflow-x-auto bg-card/40">
                                    <SignaturesView v-if="mounted" :signatures="SIGNATURES" :connections="MAP_CONNECTIONS" />
                                </div>
                            </div>
                            <div v-reveal="'right'" class="lg:order-2">
                                <h2 class="section-title">Scanning is just copy and paste</h2>
                                <p class="section-lead">
                                    Copy your probe scanner results in game, paste them in, and the map sorts it all out. New signatures get added,
                                    the ones that are gone get removed, and wormhole types line up with their connections automatically.
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
                </section>

                <!-- Section 04: Customisable widget layout -->
                <section class="feature-section feature-section--alt">
                    <div class="mx-auto max-w-7xl px-6 sm:px-12 lg:px-24">
                        <div v-reveal class="marker">
                            <span class="idx">04</span>
                            <span class="tag"><LayoutGrid class="h-3.5 w-3.5" /> Customisable layout</span>
                            <span class="line" />
                        </div>
                        <div class="grid items-center gap-16 pt-20 lg:grid-cols-2 lg:gap-24">
                            <div v-reveal="'left'">
                                <h2 class="section-title">Build the map around you</h2>
                                <p class="section-lead">
                                    The map is a grid of cards you can drag, resize, and hide. Keep the systems you watch front and centre and switch
                                    off the panels you do not use. Layouts are saved per device, with a separate arrangement for each screen size.
                                </p>
                                <ul class="points">
                                    <li><span class="dot" /> Drag and resize any card, from the map to autopilot to killmails</li>
                                    <li><span class="dot" /> Four cards stay put; eight more can be hidden and brought back any time</li>
                                    <li><span class="dot" /> Responsive breakpoints from mobile to wide desktop, each with its own layout</li>
                                </ul>
                            </div>
                            <div v-reveal="'right'" class="window">
                                <div class="window-bar">
                                    <span class="flex items-center gap-2 font-hud text-[11px] tracking-wide text-muted-foreground">
                                        <span class="size-1.5 animate-pulse rounded-full bg-empty" /> Editing layout
                                    </span>
                                    <span class="font-hud text-[10px] tracking-wide text-muted-foreground/60">J152820</span>
                                </div>
                                <div class="relative bg-card/40 p-3 pb-20">
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
                </section>

                <!-- Section 05: Access control -->
                <section class="feature-section">
                    <div class="mx-auto max-w-7xl px-6 sm:px-12 lg:px-24">
                        <div v-reveal class="marker">
                            <span class="idx">05</span>
                            <span class="tag"><ShieldCheck class="h-3.5 w-3.5" /> Access control</span>
                            <span class="line" />
                        </div>
                        <div class="grid items-center gap-16 pt-20 lg:grid-cols-2 lg:gap-24">
                            <div v-reveal="'left'" class="window lg:order-1">
                                <div class="window-bar">
                                    <span class="font-hud text-[11px] tracking-wide text-muted-foreground">Access · J152820</span>
                                </div>
                                <div class="rounded-b-xl bg-card/40 px-2 py-1">
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
                            <div v-reveal="'right'" class="lg:order-2">
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
                </section>

                <!-- Section 06: Everything else -->
                <section class="feature-section feature-section--alt">
                    <div class="mx-auto max-w-7xl px-6 sm:px-12 lg:px-24">
                        <div v-reveal class="marker">
                            <span class="idx">06</span>
                            <span class="tag"><Sparkles class="h-3.5 w-3.5" /> Everything else</span>
                            <span class="line" />
                        </div>
                        <div v-reveal="'left'" class="max-w-3xl pt-20">
                            <h2 class="section-title">Everything else you need to live in a wormhole</h2>
                            <p class="section-lead">The rest of the tools that make day-to-day wormhole life easier.</p>
                        </div>
                        <div class="mt-20 grid gap-x-12 gap-y-16 sm:grid-cols-2 lg:grid-cols-3">
                            <div v-for="feature in secondaryFeatures" :key="feature.title" v-reveal="'up'" class="group">
                                <div class="feat-icon">
                                    <component :is="feature.icon" class="h-5 w-5" :class="feature.color" />
                                </div>
                                <h3 class="mt-5 font-display text-xl font-bold text-foreground">{{ feature.title }}</h3>
                                <p class="mt-3 text-[15px] leading-7 text-muted-foreground">{{ feature.body }}</p>
                            </div>
                        </div>
                    </div>
                </section>

                <!-- CTA: full-bleed cosmic finale -->
                <section class="cta">
                    <div class="cta-core" aria-hidden="true" />
                    <div class="cta-grid" aria-hidden="true" />
                    <div v-reveal class="relative mx-auto max-w-3xl px-6 text-center sm:px-12 lg:px-24">
                        <div class="cta-eyebrow">
                            <span class="size-1.5 rounded-full bg-orange-400 shadow-[0_0_8px_var(--color-orange-400)]" />
                            Drop your first probe
                        </div>
                        <h2 class="cta-title">Ready to map the <span class="cta-void">void</span>?</h2>
                        <p class="cta-lead">Set up a map for your corp, your alliance, or just yourself, and start mapping in minutes.</p>
                        <div class="mt-12 flex justify-center">
                            <Button asChild size="lg" class="hud-btn">
                                <a :href="Eve.show().url" class="group inline-flex items-center gap-2">
                                    Start exploring
                                    <ArrowRight class="h-4 w-4 transition-transform group-hover:translate-x-0.5" />
                                </a>
                            </Button>
                        </div>
                    </div>
                </section>
            </main>

            <footer class="relative border-t border-border/60 bg-background/70 backdrop-blur-sm">
                <div class="mx-auto flex max-w-7xl flex-col items-center justify-between gap-4 px-6 py-12 sm:flex-row sm:px-12 lg:px-24">
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

.font-hud {
    font-family: var(--font-hud);
}

.text-glow {
    filter: drop-shadow(0 0 32px color-mix(in oklab, var(--color-orange-400) 55%, transparent));
}

/* Chamfered primary actions, matching the instrument-panel windows. */
:deep(.hud-btn) {
    border-radius: 2px;
    clip-path: polygon(0 0, 100% 0, 100% calc(100% - 9px), calc(100% - 9px) 100%, 0 100%);
}

/* Hero: slow drift through the gradient on the key word. */
.gradient-drift {
    background-size: 220% 220%;
    animation: gradient-drift 9s ease-in-out infinite alternate;
}

@keyframes gradient-drift {
    from {
        background-position: 0% 35%;
    }
    to {
        background-position: 100% 65%;
    }
}

/* Stat strip: a HUD tick above each figure, soft glow on the numerals. */
.stat-cell::before {
    content: '';
    position: absolute;
    top: 0;
    left: 50%;
    transform: translateX(-50%);
    height: 2px;
    width: 2.75rem;
    background: linear-gradient(to right, transparent, color-mix(in oklab, var(--color-orange-400) 80%, transparent), transparent);
}

.stat-num {
    text-shadow: 0 0 28px color-mix(in oklab, var(--color-orange-400) 35%, transparent);
}

.chip {
    display: inline-flex;
    align-items: center;
    gap: 0.6rem;
    border-left: 2px solid var(--color-orange-400);
    padding-left: 0.85rem;
    font-family: var(--font-hud);
    font-size: 11px;
    letter-spacing: 0.2em;
    text-transform: uppercase;
    color: color-mix(in oklab, var(--color-orange-300) 80%, var(--foreground));
}

/* Stronger blur band over the central content column. The rails mark its
   edges, so the sharp blur boundary reads as part of the frame. */
.center-veil {
    position: fixed;
    top: 0;
    bottom: 0;
    left: 50%;
    z-index: -6;
    width: min(100vw, 80rem);
    transform: translateX(-50%);
    pointer-events: none;
    backdrop-filter: blur(18px);
    background: color-mix(in oklab, var(--background) 22%, transparent);
}

/* Nav: HUD link + status styling. */
.nav-link {
    font-family: var(--font-hud);
    font-size: 0.72rem;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--muted-foreground);
    transition: color 0.2s ease;
}

.nav-link:hover {
    color: var(--foreground);
}

/* Structural frame: two full-height rails mark the edges of the content column
   (max-w-7xl = 80rem), with ruler ticks on their outer side, a fine diagonal
   hatch in the margins, and signal pulses travelling down the rails. */
.page-frame {
    position: fixed;
    inset: 0;
    z-index: -5;
    display: none;
    pointer-events: none;
}

@media (min-width: 90rem) {
    .page-frame {
        display: block;
    }
}

.rail {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 1px;
    background: linear-gradient(
        to bottom,
        transparent,
        color-mix(in oklab, var(--foreground) 25%, transparent) 10%,
        color-mix(in oklab, var(--foreground) 25%, transparent) 90%,
        transparent
    );
}

.rail--left {
    left: calc(50vw - 40rem);
}

.rail--right {
    right: calc(50vw - 40rem);
}

/* Ruler ticks on the outer side of each rail. */
.rail::before {
    content: '';
    position: absolute;
    top: 0;
    bottom: 0;
    width: 9px;
    background: repeating-linear-gradient(to bottom, color-mix(in oklab, var(--foreground) 45%, transparent) 0 1px, transparent 1px 48px);
    -webkit-mask-image: linear-gradient(to bottom, transparent, #000 12%, #000 88%, transparent);
    mask-image: linear-gradient(to bottom, transparent, #000 12%, #000 88%, transparent);
}

.rail--left::before {
    left: -11px;
}

.rail--right::before {
    right: -11px;
}

.frame-margin {
    position: absolute;
    top: 0;
    bottom: 0;
    width: calc(50vw - 40rem - 1rem);
    background: repeating-linear-gradient(45deg, color-mix(in oklab, var(--foreground) 9%, transparent) 0 1px, transparent 1px 12px);
}

.frame-margin--left {
    left: 0;
    -webkit-mask-image: linear-gradient(to right, transparent 10%, #000);
    mask-image: linear-gradient(to right, transparent 10%, #000);
}

.frame-margin--right {
    right: 0;
    -webkit-mask-image: linear-gradient(to left, transparent 10%, #000);
    mask-image: linear-gradient(to left, transparent 10%, #000);
}

/* Signal pulses: a short glowing dash riding each rail, like traffic on the chain. */
.rail-pulse {
    position: absolute;
    top: 0;
    left: -1.5px;
    width: 4px;
    height: 110px;
    background: linear-gradient(to bottom, transparent, color-mix(in oklab, var(--color-orange-400) 90%, transparent), transparent);
    filter: drop-shadow(0 0 6px color-mix(in oklab, var(--color-orange-400) 70%, transparent));
    animation: rail-travel 13s linear infinite;
}

.rail-pulse--alt {
    background: linear-gradient(to bottom, transparent, color-mix(in oklab, var(--color-orange-400) 90%, transparent), transparent);
    filter: drop-shadow(0 0 6px color-mix(in oklab, var(--color-orange-400) 70%, transparent));
    animation-duration: 17s;
    animation-delay: -9s;
}

@keyframes rail-travel {
    from {
        transform: translateY(-130px);
    }
    to {
        transform: translateY(100vh);
    }
}

@media (prefers-reduced-motion: reduce) {
    .rail-pulse {
        display: none;
    }
}

/* Section separation. Each section carries its accent in --sec: the marker, the
   chain connector, the ambient light, and the window chrome all key off it. */
.feature-section {
    position: relative;
    isolation: isolate;
    border-top: 1px solid color-mix(in oklab, var(--border) 70%, transparent);
    padding-block: 9rem 6rem;
}

@media (min-width: 640px) {
    .feature-section {
        padding-block: 12rem 8rem;
    }
}

.feature-section--alt {
    background: color-mix(in oklab, var(--card) 14%, transparent);
}

/* Ambient section light: a soft radial wash in the section's own wormhole-class
   colour, so each scroll stop sits in its own atmosphere. Alt sections are lit
   from the other side to keep the page from feeling stamped. */
.feature-section::before {
    content: '';
    position: absolute;
    inset: 0;
    z-index: -1;
    pointer-events: none;
    background: radial-gradient(44% 48% at 78% 20%, color-mix(in oklab, var(--sec, var(--color-orange-400)) 6%, transparent), transparent 70%);
}

.feature-section--alt::before {
    background: radial-gradient(44% 48% at 22% 20%, color-mix(in oklab, var(--sec, var(--color-orange-400)) 6%, transparent), transparent 70%);
}

/* Techy section marker: 01 // TAG ───────── (accent set per-section via --sec) */
.marker {
    position: relative;
    display: flex;
    align-items: center;
    gap: 1rem;
}

/* Chain connector: a dashed wormhole-style connection dropping into each marker,
   the same visual language the map uses for connections between systems. */
.marker::before {
    content: '';
    position: absolute;
    bottom: calc(100% + 0.6rem);
    left: clamp(0.8rem, 0.55rem + 1vw, 1.3rem);
    width: 2px;
    height: clamp(4rem, 3rem + 5vw, 9rem);
    background: repeating-linear-gradient(
        to bottom,
        color-mix(in oklab, var(--sec, var(--color-orange-400)) 75%, transparent) 0 5px,
        transparent 5px 11px
    );
    -webkit-mask-image: linear-gradient(to bottom, transparent, #000 70%);
    mask-image: linear-gradient(to bottom, transparent, #000 70%);
}

.marker .idx {
    font-family: var(--font-hud);
    font-size: clamp(1.75rem, 1.2rem + 2vw, 2.75rem);
    font-weight: 700;
    line-height: 1;
    color: var(--sec, var(--color-orange-400));
    text-shadow: 0 0 24px color-mix(in oklab, var(--sec, var(--color-orange-400)) 60%, transparent);
}

.marker .tag {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    font-family: var(--font-hud);
    font-size: 0.75rem;
    letter-spacing: 0.22em;
    text-transform: uppercase;
    color: var(--foreground);
}

.marker .tag svg {
    color: var(--sec, var(--color-orange-400));
}

.marker .line {
    height: 1px;
    flex: 1;
    background: linear-gradient(to right, color-mix(in oklab, var(--sec, var(--color-orange-400)) 65%, transparent), transparent);
}

/* Section copy: big + bold */
.section-title {
    font-family: var(--font-display);
    font-size: clamp(2.25rem, 1.5rem + 3vw, 3.75rem);
    font-weight: 800;
    line-height: 1.04;
    letter-spacing: -0.02em;
    color: var(--foreground);
}

.section-lead {
    margin-top: 1.5rem;
    max-width: 36rem;
    font-size: 1.2rem;
    line-height: 1.7;
    font-weight: 500;
    color: var(--muted-foreground);
}

.points {
    margin-top: 2.75rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
}

.points li {
    display: flex;
    align-items: flex-start;
    gap: 0.875rem;
    font-size: 1.0625rem;
    font-weight: 500;
    line-height: 1.5;
    color: color-mix(in oklab, var(--foreground) 82%, transparent);
}

.dot {
    background: var(--sec, var(--color-orange-400));
    margin-top: 0.55rem;
    height: 0.45rem;
    width: 0.45rem;
    flex-shrink: 0;
    border-radius: 9999px;
    box-shadow: 0 0 10px currentColor;
}

/* Open-source link list: hairline rows instead of cards. */
.oss-link {
    display: flex;
    align-items: flex-start;
    gap: 1rem;
    border-top: 1px solid color-mix(in oklab, var(--border) 80%, transparent);
    padding-block: 1.5rem;
    transition: border-color 0.25s ease;
}

.oss-link:last-child {
    border-bottom: 1px solid color-mix(in oklab, var(--border) 80%, transparent);
}

.oss-link:hover {
    border-top-color: color-mix(in oklab, var(--color-orange-400) 50%, var(--border));
}

/* Paste hint chip */
.paste-hint {
    margin-top: 2rem;
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    border-radius: 0.75rem;
    border: 1px dashed color-mix(in oklab, var(--color-orange-400) 45%, var(--border));
    background: color-mix(in oklab, var(--color-red-500) 7%, var(--card));
    padding: 0.6rem 0.9rem;
}

.paste-hint .kbd {
    border-radius: 0.4rem;
    border: 1px solid color-mix(in oklab, var(--border) 90%, transparent);
    border-bottom-width: 2px;
    background: var(--background);
    padding: 0.1rem 0.45rem;
    font-family: var(--font-hud);
    font-size: 0.75rem;
    font-weight: 700;
    color: var(--foreground);
}

.paste-hint .plus {
    font-family: var(--font-hud);
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
    height: 3rem;
    width: 3rem;
    align-items: center;
    justify-content: center;
    border-radius: 0.85rem;
    border: 1px solid color-mix(in oklab, var(--color-orange-400) 25%, var(--border));
    background: color-mix(in oklab, var(--card) 70%, transparent);
    backdrop-filter: blur(8px);
    transition:
        border-color 0.3s ease,
        box-shadow 0.3s ease,
        transform 0.3s ease;
}

.group:hover .feat-icon {
    border-color: color-mix(in oklab, var(--color-orange-400) 60%, var(--border));
    box-shadow: 0 0 28px -8px color-mix(in oklab, var(--color-orange-400) 60%, transparent);
    transform: translateY(-2px);
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
    border-radius: 0.5rem;
    border: 1px dashed color-mix(in oklab, var(--border) 95%, transparent);
    background: color-mix(in oklab, var(--muted) 30%, transparent);
    padding: 0.5rem 0.6rem;
    font-family: var(--font-hud);
    font-size: 0.7rem;
    letter-spacing: 0.04em;
    color: var(--muted-foreground);
}

.wg-tile::after {
    content: '';
    height: 0.5rem;
    width: 0.5rem;
    align-self: flex-end;
    border-right: 2px solid color-mix(in oklab, var(--color-orange-400) 60%, transparent);
    border-bottom: 2px solid color-mix(in oklab, var(--color-orange-400) 60%, transparent);
}

.wg-map {
    grid-column: 1 / -1;
    min-height: 5rem;
    border-style: solid;
    border-color: color-mix(in oklab, var(--color-orange-400) 40%, var(--border));
    background: color-mix(in oklab, var(--color-orange-500) 8%, transparent);
    color: color-mix(in oklab, var(--color-orange-300) 70%, var(--foreground));
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
    font-family: var(--font-hud);
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
    font-family: var(--font-hud);
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

/* App-window frame around real components, styled as a ship instrument panel:
   sharp symmetric corners, hairline accent border, bracket details — no glow. */
.window {
    position: relative;
    z-index: 1;
    overflow: hidden;
    border: 1px solid color-mix(in oklab, var(--sec, var(--color-orange-400)) 30%, var(--border));
    background: color-mix(in oklab, var(--card) 88%, transparent);
    box-shadow: inset 0 1px 0 color-mix(in oklab, var(--foreground) 6%, transparent);
    backdrop-filter: blur(12px);
}

/* Corner brackets, top-left and bottom-right. */
.window::before,
.window::after {
    content: '';
    position: absolute;
    z-index: 3;
    height: 14px;
    width: 14px;
    border: 2px solid color-mix(in oklab, var(--sec, var(--color-orange-400)) 80%, transparent);
    pointer-events: none;
}

.window::before {
    top: 0;
    left: 0;
    border-right: 0;
    border-bottom: 0;
}

.window::after {
    right: 0;
    bottom: 0;
    border-left: 0;
    border-top: 0;
}

.window-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid color-mix(in oklab, var(--sec, var(--color-orange-400)) 14%, var(--border));
    background: linear-gradient(
        to bottom,
        color-mix(in oklab, var(--sec, var(--color-orange-400)) 7%, color-mix(in oklab, var(--muted) 45%, transparent)),
        color-mix(in oklab, var(--muted) 30%, transparent)
    );
    padding: 0.7rem 0.95rem;
}

/* Hero console */
.hero-console {
    position: relative;
}

.console-glow {
    position: absolute;
    inset: -14% -10% 0 -10%;
    z-index: 0;
    background: radial-gradient(60% 60% at 62% 35%, color-mix(in oklab, var(--color-orange-400) 32%, transparent), transparent 70%);
    filter: blur(56px);
}

/* CTA: full-bleed cosmic finale */
.cta {
    position: relative;
    overflow: hidden;
    border-top: 1px solid color-mix(in oklab, var(--border) 70%, transparent);
    padding-block: 12rem;
}

@media (min-width: 640px) {
    .cta {
        padding-block: 15rem;
    }
}

.cta-core {
    position: absolute;
    left: 50%;
    top: 50%;
    height: 620px;
    width: 620px;
    transform: translate(-50%, -50%);
    border-radius: 9999px;
    background: radial-gradient(
        circle,
        color-mix(in oklab, var(--color-orange-400) 34%, transparent),
        color-mix(in oklab, var(--color-red-500) 20%, transparent) 45%,
        transparent 70%
    );
    filter: blur(56px);
    animation: cta-pulse 6s ease-in-out infinite;
    will-change: transform, opacity;
}

@keyframes cta-pulse {
    0%,
    100% {
        opacity: 0.7;
        transform: translate(-50%, -50%) scale(1);
    }
    50% {
        opacity: 1;
        transform: translate(-50%, -50%) scale(1.08);
    }
}

.cta-grid {
    position: absolute;
    inset: 0;
    background-image: linear-gradient(to right, var(--grid) 1px, transparent 1px), linear-gradient(to bottom, var(--grid) 1px, transparent 1px);
    background-size: 44px 44px;
    opacity: 0.5;
    -webkit-mask-image: radial-gradient(closest-side at 50% 50%, #000, transparent 78%);
    mask-image: radial-gradient(closest-side at 50% 50%, #000, transparent 78%);
}

.cta-eyebrow {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    font-family: var(--font-hud);
    font-size: 0.7rem;
    letter-spacing: 0.24em;
    text-transform: uppercase;
    color: color-mix(in oklab, var(--color-orange-300) 80%, var(--foreground));
}

.cta-title {
    margin-top: 1.5rem;
    font-family: var(--font-display);
    font-size: clamp(2.75rem, 1.8rem + 4vw, 5rem);
    font-weight: 800;
    line-height: 1;
    letter-spacing: -0.02em;
    color: var(--foreground);
}

.cta-void {
    background: linear-gradient(135deg, var(--color-orange-300), var(--color-orange-400), var(--color-red-500));
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
    filter: drop-shadow(0 0 36px color-mix(in oklab, var(--color-orange-400) 60%, transparent));
}

.cta-lead {
    margin-top: 1.75rem;
    margin-inline: auto;
    max-width: 34rem;
    font-size: 1.2rem;
    line-height: 1.7;
    font-weight: 500;
    color: var(--muted-foreground);
}

@media (prefers-reduced-motion: reduce) {
    .cta-core {
        animation: none;
    }
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
    .hero-console,
    .gradient-drift {
        animation: none;
    }
}
</style>
