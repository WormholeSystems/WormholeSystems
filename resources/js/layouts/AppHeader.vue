<script setup lang="ts">
import DiscordIcon from '@/components/icons/DiscordIcon.vue';
import { CharacterImage } from '@/components/images';
import Breadcrumbs from '@/components/navigation/Breadcrumbs.vue';
import ServerStatus from '@/components/ServerStatus.vue';
import { Button } from '@/components/ui/button';
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { NavigationMenu, NavigationMenuItem, NavigationMenuList, navigationMenuTriggerStyle } from '@/components/ui/navigation-menu';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from '@/components/ui/sheet';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import UserMenuContent from '@/components/UserMenuContent.vue';
import useUser from '@/composables/useUser';
import Appearance from '@/layouts/Appearance.vue';
import AppLogo from '@/layouts/AppLogo.vue';
import { home } from '@/routes';
import type { AppPageProps, BreadcrumbItem, NavItem } from '@/types';
import { TCharacter } from '@/types/models';
import { Link, usePage } from '@inertiajs/vue3';
import { AlertTriangle, LayoutGrid, Menu } from 'lucide-vue-next';
import { computed } from 'vue';

interface Props {
    breadcrumbs?: BreadcrumbItem[];
}

const props = withDefaults(defineProps<Props>(), {
    breadcrumbs: () => [],
});

const page = usePage<
    AppPageProps<{
        missing_scopes: TCharacter[];
    }>
>();
const user = useUser();

const isCurrentRoute = computed(() => (url: string) => page.url === url);

const activeItemStyles = computed(
    () => (url: string) => (isCurrentRoute.value(url) ? 'text-neutral-900 dark:bg-neutral-800 dark:text-neutral-100' : ''),
);

const mainNavItems: NavItem[] = [
    {
        title: 'Maps',
        href: '/maps',
        icon: LayoutGrid,
    },
    {
        title: 'Discord',
        href: page.props.discord.invite,
        icon: DiscordIcon,
        isExternal: true,
    },
];

const rightNavItems: NavItem[] = [];
</script>

<template>
    <div>
        <div class="border-b border-sidebar-border/80">
            <div class="grid h-16 grid-cols-[1fr_auto] items-center px-4 md:grid-cols-3">
                <!-- Left: Mobile menu + Logo + Desktop nav -->
                <div class="flex items-center gap-4">
                    <!-- Mobile Menu -->
                    <div class="lg:hidden">
                        <Sheet>
                            <SheetTrigger :as-child="true">
                                <Button variant="ghost" size="icon" class="h-9 w-9">
                                    <Menu class="h-5 w-5" />
                                </Button>
                            </SheetTrigger>
                            <SheetContent side="left" class="w-[300px] p-6">
                                <SheetTitle class="sr-only">Navigation Menu</SheetTitle>
                                <SheetHeader class="flex justify-start text-left">
                                    <AppLogo />
                                </SheetHeader>
                                <div class="flex h-full flex-1 flex-col justify-between space-y-4 py-6">
                                    <nav class="-mx-3 space-y-1">
                                        <template v-for="item in mainNavItems" :key="item.title">
                                            <Link
                                                v-if="!item.isExternal"
                                                :href="item.href"
                                                class="flex items-center gap-x-3 rounded-lg px-3 py-2 text-sm font-medium hover:bg-accent"
                                                :class="activeItemStyles(item.href)"
                                            >
                                                <component v-if="item.icon" :is="item.icon" class="h-5 w-5" />
                                                {{ item.title }}
                                            </Link>
                                            <a
                                                v-else
                                                :href="item.href"
                                                target="_blank"
                                                rel="noopener noreferrer"
                                                class="flex items-center gap-x-3 rounded-lg px-3 py-2 text-sm font-medium hover:bg-accent"
                                            >
                                                <component v-if="item.icon" :is="item.icon" class="h-5 w-5" />
                                                {{ item.title }}
                                            </a>
                                        </template>
                                    </nav>
                                    <div class="flex flex-col space-y-4">
                                        <!-- Missing scopes alert for mobile -->
                                        <div v-if="user && page.props.missing_scopes.length" class="border-t border-sidebar-border/50 pt-4">
                                            <div class="mb-2 flex items-center space-x-2">
                                                <AlertTriangle class="h-4 w-4 text-red-500" />
                                                <div class="text-xs font-medium text-red-700 dark:text-red-400">Missing ESI Scopes</div>
                                            </div>
                                            <div class="mb-2 text-xs text-muted-foreground">Characters needing re-authorization:</div>
                                            <ul class="mb-2 space-y-1">
                                                <li
                                                    v-for="character in page.props.missing_scopes"
                                                    :key="character.id"
                                                    class="flex items-center space-x-2 text-xs"
                                                >
                                                    <div class="h-1.5 w-1.5 rounded-full bg-red-500"></div>
                                                    <span class="font-medium">{{ character.name }}</span>
                                                </li>
                                            </ul>
                                            <div class="text-xs text-muted-foreground">Please re-authorize to restore functionality.</div>
                                        </div>

                                        <div class="border-t border-sidebar-border/50 pt-4">
                                            <div class="mb-2 text-xs text-muted-foreground">Server Status</div>
                                            <ServerStatus />
                                        </div>
                                        <a
                                            v-for="item in rightNavItems"
                                            :key="item.title"
                                            :href="item.href"
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            class="flex items-center space-x-2 text-sm font-medium"
                                        >
                                            <component v-if="item.icon" :is="item.icon" class="h-5 w-5" />
                                            <span>{{ item.title }}</span>
                                        </a>
                                    </div>
                                </div>
                            </SheetContent>
                        </Sheet>
                    </div>

                    <Link :href="home()" class="flex items-center gap-x-2">
                        <AppLogo />
                    </Link>

                    <!-- Desktop Menu -->
                    <div class="hidden h-full lg:flex">
                        <NavigationMenu class="flex h-full items-stretch">
                            <NavigationMenuList class="flex h-full items-stretch space-x-2">
                                <NavigationMenuItem v-for="(item, index) in mainNavItems" :key="index" class="relative flex h-full items-center">
                                    <Link
                                        v-if="!item.isExternal"
                                        :class="[navigationMenuTriggerStyle(), activeItemStyles(item.href), 'h-9 cursor-pointer px-3']"
                                        :href="item.href"
                                    >
                                        <component v-if="item.icon" :is="item.icon" class="mr-2 h-4 w-4" />
                                        {{ item.title }}
                                    </Link>
                                    <a
                                        v-else
                                        :class="[navigationMenuTriggerStyle(), activeItemStyles(item.href), 'h-9 cursor-pointer px-3']"
                                        :href="item.href"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                    >
                                        <component v-if="item.icon" :is="item.icon" class="mr-2 h-4 w-4" />
                                        {{ item.title }}
                                    </a>
                                    <div
                                        v-if="isCurrentRoute(item.href)"
                                        class="absolute bottom-0 left-0 h-0.5 w-full translate-y-px bg-black dark:bg-white"
                                    ></div>
                                </NavigationMenuItem>
                            </NavigationMenuList>
                        </NavigationMenu>
                    </div>
                </div>

                <!-- Center: Server Status (desktop only) -->
                <div class="hidden justify-center md:flex">
                    <ServerStatus />
                </div>

                <!-- Right: Actions -->
                <div class="flex items-center justify-end space-x-1 sm:space-x-2 md:col-start-3">
                    <!-- Missing scopes alert -->
                    <template v-if="user && page.props.missing_scopes.length">
                        <Popover>
                            <PopoverTrigger as-child>
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    class="h-9 w-9 text-red-500 hover:bg-red-50 hover:text-red-600 dark:hover:bg-red-950/20"
                                >
                                    <span class="sr-only">Missing ESI scopes</span>
                                    <AlertTriangle class="h-4 w-4" />
                                </Button>
                            </PopoverTrigger>
                            <PopoverContent class="w-80" align="end">
                                <div class="space-y-3">
                                    <div class="flex items-center space-x-2">
                                        <AlertTriangle class="h-5 w-5 text-red-500" />
                                        <h4 class="font-semibold text-red-700 dark:text-red-400">Missing ESI Scopes</h4>
                                    </div>
                                    <div class="text-sm text-muted-foreground">
                                        <p class="mb-2">The following characters need to be re-authorized with updated permissions:</p>
                                        <ul class="space-y-1">
                                            <li
                                                v-for="character in page.props.missing_scopes"
                                                :key="character.id"
                                                class="flex items-center space-x-2"
                                            >
                                                <div class="h-2 w-2 rounded-full bg-red-500"></div>
                                                <span class="font-medium">{{ character.name }}</span>
                                            </li>
                                        </ul>
                                        <p class="mt-3 text-xs text-muted-foreground">
                                            Please re-authorize these characters to restore full functionality.
                                        </p>
                                    </div>
                                </div>
                            </PopoverContent>
                        </Popover>
                    </template>

                    <!-- Right nav items (desktop only) -->
                    <div class="hidden items-center space-x-1 lg:flex">
                        <template v-for="item in rightNavItems" :key="item.title">
                            <TooltipProvider :delay-duration="0">
                                <Tooltip>
                                    <TooltipTrigger>
                                        <Button variant="ghost" size="icon" as-child class="group h-9 w-9 cursor-pointer">
                                            <a :href="item.href" target="_blank" rel="noopener noreferrer">
                                                <span class="sr-only">{{ item.title }}</span>
                                                <component :is="item.icon" class="size-5 opacity-80 group-hover:opacity-100" />
                                            </a>
                                        </Button>
                                    </TooltipTrigger>
                                    <TooltipContent>
                                        <p>{{ item.title }}</p>
                                    </TooltipContent>
                                </Tooltip>
                            </TooltipProvider>
                        </template>
                    </div>

                    <Appearance />

                    <!-- User menu -->
                    <template v-if="user">
                        <DropdownMenu>
                            <DropdownMenuTrigger :as-child="true">
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    class="relative h-9 w-9 rounded-lg p-1 focus-within:ring-2 focus-within:ring-primary"
                                >
                                    <CharacterImage
                                        :character_id="user.active_character.id"
                                        :character_name="user.active_character.name"
                                        class="h-7 w-7 rounded-lg"
                                    />
                                </Button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent align="end" class="w-56">
                                <UserMenuContent :user="user" />
                            </DropdownMenuContent>
                        </DropdownMenu>
                    </template>
                </div>
            </div>
        </div>

        <div v-if="props.breadcrumbs.length > 1" class="flex w-full border-b border-sidebar-border/70">
            <div class="mx-auto flex h-12 w-full items-center justify-start px-4 text-neutral-500 md:max-w-7xl">
                <Breadcrumbs :breadcrumbs="breadcrumbs" />
            </div>
        </div>
    </div>
</template>
