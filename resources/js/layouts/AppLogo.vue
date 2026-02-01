<script setup lang="ts">
import Logo from '@/components/icons/Logo.vue';
import { Badge } from '@/components/ui/badge';
</script>

<template>
    <Logo class="size-6 fill-current text-black dark:text-white" />
    <div class="ml-1 grid flex-1 text-left text-sm">
        <span class="mb-0.5 truncate leading-tight font-semibold"> WormholeSystems <Badge variant="outline" class="ml-2">beta</Badge></span>
    </div>
</template>
