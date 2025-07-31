<script setup lang="ts">
import TelescopeIcon from '@/components/icons/TelescopeIcon.vue';
import Notifications from '@/components/Notifications.vue';
import { Button } from '@/components/ui/button';
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import useUser from '@/composables/useUser';
import { home } from '@/routes';
import Eve from '@/routes/eve';
import { Head, Link } from '@inertiajs/vue3';
import { format } from 'date-fns';
import { MapIcon, RouteIcon, ShieldCheckIcon, UsersIcon } from 'lucide-vue-next';

const currentYear = format(new Date(), 'yyyy');
const user = useUser();
</script>

<template>
    <div class="flex min-h-screen flex-col bg-gradient-to-br from-background via-muted to-background">
        <Head title="WormholeSystems - EVE Online Mapping" />

        <!-- Header -->
        <header class="container mx-auto px-4 py-6">
            <div class="flex items-center gap-4">
                <div class="flex h-12 w-12 items-center justify-center rounded-lg border bg-accent">
                    <TelescopeIcon class="size-8 text-foreground" />
                </div>
                <div class="text-2xl font-bold text-foreground">WormholeSystems</div>
            </div>
        </header>

        <!-- Main Content -->
        <main class="flex-1">
            <!-- Hero Section -->
            <section class="container mx-auto px-4 py-20 text-center">
                <div class="mx-auto max-w-4xl">
                    <h1 class="mb-6 text-5xl font-bold text-foreground md:text-6xl">Navigate the Unknown</h1>
                    <p class="mb-8 text-xl text-muted-foreground md:text-2xl">
                        The premier mapping tool for EVE Online wormhole exploration and fleet coordination
                    </p>
                    <div class="flex flex-col gap-4 sm:flex-row sm:justify-center">
                        <Button asChild size="lg" v-if="!user">
                            <a :href="Eve.show().url" rel="noopener noreferrer">Get Started</a>
                        </Button>
                        <Button as-child size="lg" v-else>
                            <Link :href="home()">Go to maps</Link>
                        </Button>
                        <Button asChild variant="secondary" size="lg">
                            <a href="#features">Learn More</a>
                        </Button>
                    </div>
                </div>
            </section>

            <!-- Features Section -->
            <section id="features" class="container mx-auto px-4 py-20">
                <div class="mx-auto max-w-6xl">
                    <h2 class="mb-12 text-center text-3xl font-bold text-foreground">Everything you need for wormhole exploration</h2>
                    <div class="grid gap-8 md:grid-cols-2 lg:grid-cols-4">
                        <Card>
                            <CardHeader>
                                <div class="mb-4 flex h-12 w-12 items-center justify-center rounded-lg bg-indigo-600">
                                    <MapIcon class="h-6 w-6 text-white" />
                                </div>
                                <CardTitle class="text-card-foreground">Interactive Maps</CardTitle>
                                <CardDescription class="text-muted-foreground">
                                    Create and manage detailed wormhole maps with real-time updates
                                </CardDescription>
                            </CardHeader>
                        </Card>

                        <Card>
                            <CardHeader>
                                <div class="mb-4 flex h-12 w-12 items-center justify-center rounded-lg bg-emerald-600">
                                    <UsersIcon class="h-6 w-6 text-white" />
                                </div>
                                <CardTitle class="text-card-foreground">Team Collaboration</CardTitle>
                                <CardDescription class="text-muted-foreground">
                                    Share maps with your corporation and alliance members
                                </CardDescription>
                            </CardHeader>
                        </Card>

                        <Card>
                            <CardHeader>
                                <div class="mb-4 flex h-12 w-12 items-center justify-center rounded-lg bg-orange-600">
                                    <RouteIcon class="h-6 w-6 text-white" />
                                </div>
                                <CardTitle class="text-card-foreground">Route Planning</CardTitle>
                                <CardDescription class="text-muted-foreground"> Plan optimal routes through wormhole chains </CardDescription>
                            </CardHeader>
                        </Card>

                        <Card>
                            <CardHeader>
                                <div class="mb-4 flex h-12 w-12 items-center justify-center rounded-lg bg-red-600">
                                    <ShieldCheckIcon class="h-6 w-6 text-white" />
                                </div>
                                <CardTitle class="text-card-foreground">Security Status</CardTitle>
                                <CardDescription class="text-muted-foreground"> Track system security and activity in real-time </CardDescription>
                            </CardHeader>
                        </Card>
                    </div>
                </div>
            </section>

            <!-- CTA Section -->
            <section class="container mx-auto px-4 py-20">
                <div class="mx-auto max-w-4xl text-center">
                    <h2 class="mb-4 text-3xl font-bold text-foreground">Ready to explore the unknown?</h2>
                    <p class="mb-8 text-xl text-muted-foreground">Join all capsuleers who trust WormholeSystems for their wormhole operations</p>
                    <Button asChild size="lg">
                        <a :href="Eve.show().url" rel="noopener noreferrer"> Get Started Now</a>
                    </Button>
                </div>
            </section>
        </main>

        <!-- Footer -->
        <footer class="border border-t bg-secondary/50 backdrop-blur-sm">
            <div class="container mx-auto px-4 py-8">
                <div class="flex flex-col items-center gap-4 sm:flex-row sm:justify-between">
                    <div class="flex items-center gap-2">
                        <TelescopeIcon class="size-6 text-muted-foreground" />
                        <span class="text-muted-foreground">WormholeSystems</span>
                    </div>
                    <div class="text-sm text-muted-foreground">
                        © {{ currentYear }} WormholeSystems. EVE Online and the EVE logo are trademarks of CCP hf.
                    </div>
                </div>
            </div>
        </footer>
    </div>
    <Notifications />
</template>
