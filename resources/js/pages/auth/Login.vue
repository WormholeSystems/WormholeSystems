<script setup lang="ts">
import LockIcon from '@/components/icons/LockIcon.vue';
import TelescopeIcon from '@/components/icons/TelescopeIcon.vue';
import Notifications from '@/components/Notifications.vue';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import Eve from '@/routes/eve';
import { UTCDate } from '@date-fns/utc';
import { Head } from '@inertiajs/vue3';
import { format } from 'date-fns';

const currentYear = format(new UTCDate(), 'yyyy');
</script>

<template>
    <div class="flex min-h-screen flex-col items-center justify-center bg-gradient-to-br from-background via-muted to-background p-6">
        <Head title="Login" />

        <div class="w-full max-w-md">
            <!-- Logo and Brand -->
            <div class="mb-8 flex flex-col items-center gap-4">
                <div class="flex h-20 w-20 items-center justify-center rounded-xl border bg-accent">
                    <TelescopeIcon class="text-4xl text-foreground" />
                </div>
                <div class="text-center">
                    <h1 class="text-2xl font-bold text-foreground">WormholeSystems</h1>
                    <p class="text-sm text-muted-foreground">EVE Online Wormhole Mapping</p>
                </div>
            </div>

            <!-- Login Card -->
            <Card class="border shadow-lg">
                <CardHeader class="text-center">
                    <CardTitle class="text-xl">Join the Fleet</CardTitle>
                    <CardDescription> Sign in or create your account with EVE Online to start mapping wormhole systems </CardDescription>
                </CardHeader>
                <CardContent class="space-y-6">
                    <!-- EVE Online SSO Button -->
                    <Button asChild class="w-full" size="lg">
                        <a :href="Eve.show().url" class="flex items-center justify-center gap-3">
                            <LockIcon />
                            Sign in with EVE Online
                        </a>
                    </Button>

                    <!-- Info Section -->
                    <div class="text-center text-sm text-muted-foreground">
                        <p>Secure authentication through CCP's official EVE Online SSO</p>
                    </div>
                </CardContent>
            </Card>

            <!-- Footer -->
            <div class="mt-8 text-center text-xs text-muted-foreground">
                <p>© {{ currentYear }} WormholeSystems</p>
                <p class="mt-1">EVE Online and the EVE logo are trademarks of CCP hf.</p>
            </div>
        </div>
    </div>
    <Notifications />
</template>

<style scoped></style>
