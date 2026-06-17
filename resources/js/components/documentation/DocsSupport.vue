<script setup lang="ts">
import { Button } from '@/components/ui/button';
import { Check, Coins, Copy, ExternalLink, Github, Sparkles } from 'lucide-vue-next';
import { ref } from 'vue';

const DONATION_CHARACTER = 'MutaMate';

const copied = ref(false);

async function copyCharacter(): Promise<void> {
    await navigator.clipboard.writeText(DONATION_CHARACTER);
    copied.value = true;
    window.setTimeout(() => (copied.value = false), 2000);
}
</script>

<template>
    <div class="rounded-xl border border-border/50 bg-card p-5">
        <h3 class="text-xs font-semibold tracking-wide text-muted-foreground/70 uppercase">Support WormholeSystems</h3>
        <p class="mt-2.5 text-sm leading-6 text-muted-foreground">
            Free &amp; open source — donations keep the servers running and fund new features.
        </p>

        <!-- Donate ISK -->
        <div class="mt-5">
            <div class="flex items-center gap-2 text-sm font-semibold text-foreground">
                <Coins class="size-4 text-amber-400" />
                Donate ISK in-game
            </div>
            <p class="mt-2 text-xs leading-5 text-muted-foreground">Send ISK to this character — click to copy:</p>

            <button
                type="button"
                class="group mt-2 flex w-full items-center justify-between gap-3 rounded-lg border border-border bg-background px-3.5 py-2.5 text-left transition-colors hover:border-amber-400/60 hover:bg-amber-500/5"
                :aria-label="`Copy character name ${DONATION_CHARACTER}`"
                @click="copyCharacter"
            >
                <span class="truncate font-mono text-sm font-semibold tracking-tight text-foreground">{{ DONATION_CHARACTER }}</span>
                <span
                    class="flex shrink-0 items-center gap-1 text-xs font-medium transition-colors"
                    :class="copied ? 'text-emerald-400' : 'text-muted-foreground group-hover:text-foreground'"
                >
                    <component :is="copied ? Check : Copy" class="size-3.5" />
                    {{ copied ? 'Copied' : 'Copy' }}
                </span>
            </button>

            <p class="mt-2.5 flex items-start gap-1.5 text-xs leading-5 text-muted-foreground">
                <Sparkles class="mt-0.5 size-3.5 shrink-0 text-amber-400" />
                <span>
                    Includes a free
                    <a
                        href="https://mutamarket.com"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="font-medium text-foreground underline underline-offset-2 hover:text-amber-400"
                        >MutaMarket<ExternalLink class="ml-0.5 inline size-2.5 align-baseline"
                    /></a>
                    premium membership.
                </span>
            </p>
        </div>

        <div class="my-5 h-px bg-border/50" />

        <!-- GitHub Sponsors -->
        <Button as-child variant="outline" size="sm" class="w-full">
            <a href="https://github.com/sponsors/wormholesystems" target="_blank" rel="noopener noreferrer" class="inline-flex items-center gap-2">
                <Github class="size-4" />
                Sponsor on GitHub
                <ExternalLink class="size-3" />
            </a>
        </Button>
    </div>
</template>
