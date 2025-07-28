<script setup lang="ts">
import TelescopeIcon from '@/components/icons/TelescopeIcon.vue';
import { Button } from '@/components/ui/button';
import { Head, Link, usePage } from '@inertiajs/vue3';
import { HomeIcon, MessageCircleIcon } from 'lucide-vue-next';

interface Props {
    status: number;
    title: string;
    message: string;
    pageTitle: string;
}

defineProps<Props>();

const page = usePage();
const discordInvite = page.props.discord_invite as string | undefined;
</script>

<template>
    <div class="flex min-h-screen items-center justify-center bg-background px-4">
        <Head :title="pageTitle" />

        <div class="w-full max-w-md text-center">
            <!-- Logo -->
            <div class="mb-8 flex items-center justify-center">
                <div class="flex h-12 w-12 items-center justify-center rounded-lg bg-accent">
                    <TelescopeIcon class="size-7 text-foreground" />
                </div>
            </div>

            <!-- Error Content -->
            <h1 class="mb-2 text-6xl font-bold text-muted-foreground">{{ status }}</h1>
            <h2 class="mb-6 text-xl font-semibold text-foreground">{{ title }}</h2>
            <p class="mb-8 text-sm text-muted-foreground">{{ message }}</p>

            <div class="flex flex-col gap-3">
                <Button as-child>
                    <Link :href="route('home')" class="flex items-center justify-center gap-2">
                        <HomeIcon class="h-4 w-4" />
                        Home
                    </Link>
                </Button>
                <Button as-child variant="outline" v-if="discordInvite">
                    <a :href="discordInvite" target="_blank" rel="noopener noreferrer" class="flex items-center justify-center gap-2">
                        <MessageCircleIcon class="h-4 w-4" />
                        Discord
                    </a>
                </Button>
            </div>
        </div>
    </div>
</template>
