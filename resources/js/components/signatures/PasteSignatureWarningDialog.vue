<script setup lang="ts">
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { TSelectedMapSolarsystem } from '@/pages/maps';
import { TCharacter } from '@/types/models';

const { targetSystem, character } = defineProps<{
    targetSystem: TSelectedMapSolarsystem | null;
    character: TCharacter | undefined;
}>();

const open = defineModel<boolean>('open', { required: true });

const emit = defineEmits<{
    confirm: [];
    cancel: [];
}>();

function handleConfirm() {
    emit('confirm');
}

function handleCancel() {
    emit('cancel');
}

function handleOpenChange(isOpen: boolean) {
    if (!isOpen) {
        emit('cancel');
    }
}
</script>

<template>
    <Dialog v-model:open="open" @update:open="handleOpenChange">
        <DialogContent v-if="targetSystem && character" class="max-w-md">
            <DialogHeader>
                <DialogTitle>System Mismatch Warning</DialogTitle>
                <DialogDescription>
                    You are pasting signatures into
                    <strong class="text-foreground">{{ targetSystem.solarsystem.name }}</strong
                    >, but your tracked character is currently in
                    <strong class="text-foreground">{{ character.status?.solarsystem?.name || 'Unknown' }}</strong
                    >.
                </DialogDescription>
            </DialogHeader>

            <div class="rounded-lg border border-yellow-500/20 bg-yellow-500/10 p-4">
                <p class="text-sm text-foreground">
                    Are you sure you want to paste signatures into a different system than where your character is located?
                </p>
            </div>

            <DialogFooter class="gap-2">
                <Button variant="outline" @click="handleCancel">Cancel</Button>
                <Button variant="default" @click="handleConfirm">Paste Anyway</Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>
</template>
