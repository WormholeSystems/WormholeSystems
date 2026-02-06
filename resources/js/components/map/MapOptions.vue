<script setup lang="ts">
import BackgroundImageIcon from '@/components/icons/BackgroundImageIcon.vue';
import CheckIcon from '@/components/icons/CheckIcon.vue';
import MinusIcon from '@/components/icons/MinusIcon.vue';
import PlusIcon from '@/components/icons/PlusIcon.vue';
import TimesIcon from '@/components/icons/TimesIcon.vue';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { useMapScale } from '@/composables/map';
import { useMapBackground } from '@/composables/useMapBackground';
import { ref } from 'vue';

const { scale, setScale } = useMapScale();

const { backgroundImageUrl, setBackgroundImageUrl, clearBackgroundImage } = useMapBackground();

const backgroundInputUrl = ref('');

function applyBackgroundImage() {
    if (backgroundInputUrl.value.trim()) {
        setBackgroundImageUrl(backgroundInputUrl.value.trim());
        backgroundInputUrl.value = '';
    }
}

function removeBackgroundImage() {
    clearBackgroundImage();
}
</script>

<template>
    <div class="absolute right-3 bottom-3 z-30 flex items-center gap-0.5 rounded-full bg-white/60 px-1 dark:bg-neutral-800/60">
        <Popover>
            <PopoverTrigger as-child>
                <Button variant="ghost" size="icon" class="h-8 w-8 rounded-full text-neutral-600 dark:text-neutral-400" title="Background Image">
                    <BackgroundImageIcon />
                </Button>
            </PopoverTrigger>
            <PopoverContent class="w-80" side="top" align="end">
                <div class="flex gap-2">
                    <Input v-model="backgroundInputUrl" placeholder="Enter image URL..." class="flex-1" @keydown.enter="applyBackgroundImage" />
                    <Button variant="ghost" size="icon" @click="applyBackgroundImage" :disabled="!backgroundInputUrl.trim()" title="Apply">
                        <CheckIcon />
                    </Button>
                    <Button variant="ghost" size="icon" @click="removeBackgroundImage" :disabled="!backgroundImageUrl" title="Clear">
                        <TimesIcon />
                    </Button>
                </div>
            </PopoverContent>
        </Popover>
        <span class="flex h-8 items-center px-1 text-neutral-600 dark:text-neutral-400">{{ (scale * 100).toFixed(0) + '%' }}</span>
        <Button variant="ghost" size="icon" class="h-8 w-8 rounded-full text-neutral-600 dark:text-neutral-400" @click="setScale(scale - 0.1)">
            <MinusIcon />
        </Button>
        <Button variant="ghost" size="icon" class="h-8 w-8 rounded-full text-neutral-600 dark:text-neutral-400" @click="setScale(scale + 0.1)">
            <PlusIcon />
        </Button>
    </div>
</template>

<style scoped></style>
