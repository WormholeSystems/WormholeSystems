<script setup lang="ts">
import DocsSupport from '@/components/documentation/DocsSupport.vue';
import DiscordIcon from '@/components/icons/DiscordIcon.vue';
import { Button } from '@/components/ui/button';
import AppLayout from '@/layouts/AppLayout.vue';
import SeoHead from '@/layouts/SeoHead.vue';
import { Link, router, usePage } from '@inertiajs/vue3';
import { Check, ChevronLeft, ChevronRight, Copy, ExternalLink, Github } from 'lucide-vue-next';
import { ref } from 'vue';

interface NavPage {
    title: string;
    slug: string;
    url: string;
}

interface NavCategory {
    title: string;
    pages: NavPage[];
}

interface DocPage {
    title: string;
    category: string;
    slug: string;
    html: string;
    markdown: string;
    edit_url: string;
    prev: NavPage | null;
    next: NavPage | null;
}

const props = defineProps<{
    navigation: NavCategory[];
    page: DocPage;
}>();

const inertiaPage = usePage();

function onMobileNavigate(event: Event): void {
    const url = (event.target as HTMLSelectElement).value;

    if (url) {
        router.visit(url, { only: ['page'], preserveState: true });
    }
}

const copied = ref(false);

async function copyMarkdown(): Promise<void> {
    await navigator.clipboard.writeText(props.page.markdown);
    copied.value = true;
    window.setTimeout(() => (copied.value = false), 2000);
}
</script>

<template>
    <AppLayout>
        <SeoHead
            :title="`${page.title} — Documentation — WormholeSystems`"
            description="Learn how WormholeSystems works — signatures, bookmarking, access control, autopilot routing, character tracking, and more."
            keywords="documentation, help, guide, wormhole, mapping, eve online"
        />

        <div class="mx-auto max-w-[84rem] px-6 py-10">
            <div class="grid gap-x-20 gap-y-12 lg:grid-cols-[240px_minmax(0,1fr)] xl:grid-cols-[256px_minmax(0,1fr)_280px]">
                <!-- Sidebar (desktop) -->
                <aside class="hidden lg:block">
                    <nav class="sticky top-20 max-h-[calc(100vh-6rem)] space-y-6 overflow-y-auto pr-2 pb-6">
                        <div v-for="category in navigation" :key="category.title" class="space-y-1">
                            <p class="px-3 pb-1 text-xs font-semibold tracking-wide text-muted-foreground/70 uppercase">
                                {{ category.title }}
                            </p>
                            <Link
                                v-for="item in category.pages"
                                :key="item.slug"
                                :href="item.url"
                                :only="['page']"
                                preserve-state
                                prefetch
                                class="block rounded-md px-3 py-1.5 text-sm transition-colors"
                                :class="
                                    item.slug === page.slug
                                        ? 'bg-muted font-medium text-foreground'
                                        : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
                                "
                            >
                                {{ item.title }}
                            </Link>
                        </div>
                    </nav>
                </aside>

                <!-- Content -->
                <div class="min-w-0">
                    <!-- Mobile page picker -->
                    <div class="mb-6 lg:hidden">
                        <label for="help-nav" class="mb-1.5 block text-xs font-medium text-muted-foreground">Browse the guide</label>
                        <select
                            id="help-nav"
                            :value="`/documentation/${page.slug}`"
                            class="w-full rounded-md border border-border bg-card px-3 py-2 text-sm text-foreground"
                            @change="onMobileNavigate"
                        >
                            <optgroup v-for="category in navigation" :key="category.title" :label="category.title">
                                <option v-for="item in category.pages" :key="item.slug" :value="item.url">
                                    {{ item.title }}
                                </option>
                            </optgroup>
                        </select>
                    </div>

                    <!-- Breadcrumb + page actions -->
                    <div class="mb-4 flex items-start justify-between gap-4">
                        <p class="text-xs font-medium tracking-wide text-muted-foreground/70 uppercase">
                            {{ page.category }}
                        </p>
                        <div class="flex shrink-0 items-center gap-2">
                            <Button variant="outline" size="sm" class="gap-1.5" @click="copyMarkdown">
                                <component :is="copied ? Check : Copy" class="size-3.5" />
                                {{ copied ? 'Copied' : 'Copy as Markdown' }}
                            </Button>
                            <Button as-child variant="outline" size="sm" class="gap-1.5">
                                <a :href="page.edit_url" target="_blank" rel="noopener noreferrer">
                                    <Github class="size-3.5" />
                                    Edit on GitHub
                                </a>
                            </Button>
                        </div>
                    </div>

                    <!-- Rendered markdown -->
                    <article
                        class="prose prose-sm max-w-none dark:prose-invert prose-headings:scroll-mt-20 prose-headings:font-semibold prose-a:text-foreground prose-a:underline-offset-4 prose-code:rounded prose-code:bg-muted prose-code:px-1 prose-code:py-0.5 prose-code:text-xs prose-code:font-normal prose-code:before:content-none prose-code:after:content-none prose-th:text-left"
                        v-html="page.html"
                    />

                    <!-- Prev / next -->
                    <nav class="mt-12 grid gap-3 border-t border-border/50 pt-6 sm:grid-cols-2">
                        <Link
                            v-if="page.prev"
                            :href="page.prev.url"
                            :only="['page']"
                            preserve-state
                            prefetch
                            class="group flex flex-col rounded-lg border border-border/50 bg-card p-4 transition-colors hover:border-border sm:items-start"
                        >
                            <span class="flex items-center gap-1 text-xs text-muted-foreground"><ChevronLeft class="size-3" /> Previous</span>
                            <span class="mt-1 text-sm font-medium group-hover:text-foreground">{{ page.prev.title }}</span>
                        </Link>
                        <div v-else class="hidden sm:block"></div>

                        <Link
                            v-if="page.next"
                            :href="page.next.url"
                            :only="['page']"
                            preserve-state
                            prefetch
                            class="group flex flex-col rounded-lg border border-border/50 bg-card p-4 text-right transition-colors hover:border-border sm:items-end"
                        >
                            <span class="flex items-center gap-1 text-xs text-muted-foreground">Next <ChevronRight class="size-3" /></span>
                            <span class="mt-1 text-sm font-medium group-hover:text-foreground">{{ page.next.title }}</span>
                        </Link>
                    </nav>

                    <!-- Docs CTA -->
                    <div class="mt-10 rounded-lg border border-border/50 bg-card p-5">
                        <div class="flex items-start gap-4">
                            <div class="rounded-lg bg-muted p-2.5">
                                <DiscordIcon class="size-5 text-foreground" />
                            </div>
                            <div class="flex-1 space-y-2">
                                <h2 class="text-sm font-semibold">Didn't find your answer?</h2>
                                <p class="text-sm text-muted-foreground">Ask in our Discord — the community and developers are happy to help.</p>
                                <Button as-child size="sm" class="mt-1">
                                    <a
                                        :href="inertiaPage.props.discord.invite"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="inline-flex items-center gap-2"
                                    >
                                        <DiscordIcon class="size-4" />
                                        Join Discord
                                        <ExternalLink class="size-3" />
                                    </a>
                                </Button>
                            </div>
                        </div>
                    </div>

                    <!-- Support (inline fallback below the xl right rail) -->
                    <DocsSupport class="mt-6 xl:hidden" />
                </div>

                <!-- Right rail: support -->
                <aside class="hidden xl:block">
                    <div class="sticky top-20">
                        <DocsSupport />
                    </div>
                </aside>
            </div>
        </div>
    </AppLayout>
</template>
