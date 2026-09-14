import { TLayout } from '@/composables/useLayout';
import { TCharacter, TServerStatus } from '@/types/models';
import type { LucideIcon } from 'lucide-vue-next';
import { Component } from 'vue';
import type { Config } from 'ziggy-js';

export interface Auth {
    user: User;
}

export interface BreadcrumbItem {
    title: string;
    href: string;
}

export interface NavItem {
    title: string;
    href: string;
    icon?: LucideIcon | Component;
    isActive?: boolean;
    isExternal?: boolean;
}

export type AppPageProps<T extends Record<string, unknown> = Record<string, unknown>> = T & {
    name: string;
    app_url: string;
    announcement: TAnnouncement | null;
    quote: { message: string; author: string };
    auth: Auth;
    ziggy: Config & { location: string };
    sidebarOpen: boolean;
    discord: {
        invite: string;
    };
    layout: TLayout;
    server_status: TServerStatus;
    sort_preferences: {
        signatures: SortPreference;
    };
    missing_scopes: TCharacter[];
    pinned_maps: TPinnedMap[];
};

export type TAnnouncementLevel = 'info' | 'warning' | 'critical';

export type TAnnouncement = {
    id: string;
    level: TAnnouncementLevel;
    title: string | null;
    message: string;
    link: { url: string; label: string } | null;
    dismissible: boolean;
};

export type TPinnedMap = {
    id: number;
    name: string;
    slug: string;
};

export interface User {
    id: number;
    name: string;
    characters: TCharacter[];
    active_character: TCharacter;
    created_at: string;
    updated_at: string;
}

export type SortPreference = {
    column: string;
    direction: 'asc' | 'desc';
};

export type BreadcrumbItemType = BreadcrumbItem;
