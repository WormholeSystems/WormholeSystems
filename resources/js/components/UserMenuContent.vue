<script setup lang="ts">
import UserInfo from '@/components/UserInfo.vue';
import MinusIcon from '@/components/icons/MinusIcon.vue';
import PlusIcon from '@/components/icons/PlusIcon.vue';
import { CharacterImage } from '@/components/images';
import { DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator } from '@/components/ui/dropdown-menu';
import { auth, logout } from '@/routes';
import UserCharacters from '@/routes/user-characters';
import type { User } from '@/types';
import { Link, router } from '@inertiajs/vue3';
import { LogOut } from 'lucide-vue-next';

interface Props {
    user: User;
}

const handleLogout = () => {
    router.flushAll();
};

defineProps<Props>();
</script>

<template>
    <DropdownMenuLabel class="p-0 font-normal">
        <div class="flex items-center gap-2 px-1 py-1.5 text-left text-sm">
            <UserInfo :user="user" :show-email="true" />
        </div>
    </DropdownMenuLabel>
    <DropdownMenuSeparator />
    <DropdownMenuGroup>
        <DropdownMenuItem v-for="character in user.characters" :key="character.id" as-child>
            <Link class="block w-full" :href="UserCharacters.update(character.id)" as="button" method="put">
                <CharacterImage :character_id="character.id" :character_name="character.name" class="mr-2 h-4 w-4" />
                {{ character.name }}
            </Link>
        </DropdownMenuItem>
        <DropdownMenuItem as-child>
            <a
                class="block w-full"
                :href="
                    auth({
                        query: {
                            add_to_account: true,
                        },
                    }).url
                "
            >
                <PlusIcon class="mr-2 h-4 w-4" />
                Add Character
            </a>
        </DropdownMenuItem>
        <DropdownMenuItem as-child v-if="user.characters.length > 1">
            <Link class="block w-full" :href="UserCharacters.delete(user.active_character.id)" method="delete" as="button">
                <MinusIcon class="mr-2 h-4 w-4" />
                Remove Character
            </Link>
        </DropdownMenuItem>
    </DropdownMenuGroup>
    <DropdownMenuSeparator />
    <DropdownMenuItem :as-child="true">
        <Link class="block w-full" method="delete" :href="logout()" @click="handleLogout" as="button">
            <LogOut class="mr-2 h-4 w-4" />
            Log out
        </Link>
    </DropdownMenuItem>
</template>
