<script setup lang="ts">
import TokenController from '@/actions/App/Http/Controllers/TokenManagementController';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import AppLayout from '@/layouts/AppLayout.vue';
import { TToken } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import { router, useForm } from '@inertiajs/vue3';
import { format } from 'date-fns';
import { toast } from 'vue-sonner';

const { tokens: tokenList, token } = defineProps<{
    tokens: TToken[];
    token: string | null;
}>();

const form = useForm({
    name: '',
});

function handleSubmit(): void {
    form.submit(TokenController.store());
}

function handleDelete(token: TToken) {
    router.delete(TokenController.destroy(token.id));
}

function handleCopy() {
    navigator.clipboard
        .writeText(token!)
        .then(() => {
            toast.success('Token copied to clipboard');
        })
        .catch(() => {
            toast.error('Failed to copy token');
        });
}

function formatDate(date: string): string {
    return format(new UTCDate(date), 'PPPpp');
}
</script>

<template>
    <AppLayout>
        <div class="container mx-auto space-y-6 p-6">
            <div class="mb-8">
                <h1 class="text-3xl font-bold">API Token Management</h1>
                <p class="mt-2 max-w-xl text-muted-foreground">
                    You can create api tokens for your account. The tokens allow you to read and modify any maps that you have access to. Be careful
                    with these tokens, as they can be used to access your data without further authentication.
                </p>
            </div>
            <Card>
                <CardHeader>
                    <CardTitle> Create New API Token </CardTitle>
                    <CardDescription>
                        Use the form below to create a new API token. Make sure to copy it immediately after creation, as it will not be shown again.
                    </CardDescription>
                </CardHeader>
                <CardContent>
                    <form @submit.prevent="handleSubmit">
                        <div class="grid grid-cols-[1fr_auto] items-center gap-2">
                            <Label for="name" class="col-span-full">Token Name</Label>
                            <Input class="col-span-1" id="name" v-model="form.name" placeholder="Enter a descriptive name for your token" required />
                            <Button type="submit" :disabled="form.processing" class="w-full"> Create Token </Button>
                            <div v-if="form.errors.name" class="text-sm text-destructive">
                                {{ form.errors.name }}
                            </div>
                        </div>
                    </form>
                    <div v-if="token" class="mt-8 grid grid-cols-[1fr_auto] items-center gap-2 rounded-lg border p-6">
                        <Label class="col-span-full">Generated Token</Label>
                        <span class="block rounded-lg border bg-muted p-2 font-mono">{{ token }}</span>
                        <Button @click="handleCopy"> Copy Token </Button>
                        <p class="col-span-full mt-2 text-sm text-muted-foreground">Keep this token secure. It will not be shown again.</p>
                    </div>
                </CardContent>
            </Card>
            <Card>
                <CardHeader>
                    <CardTitle>Existing Tokens</CardTitle>
                    <CardDescription> Below is a list of your existing API tokens. You can revoke any token at any time. </CardDescription>
                </CardHeader>
                <CardContent class="grid auto-cols-auto">
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead> Token Name </TableHead>
                                <TableHead> Last Used </TableHead>
                                <TableHead class="text-right"> Actions </TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            <TableRow v-for="token in tokenList" :key="token.id">
                                <TableCell class="font-medium">{{ token.name }}</TableCell>
                                <TableCell>
                                    {{ token.last_used_at ? formatDate(token.last_used_at) : 'Never' }}
                                </TableCell>
                                <TableCell class="text-right">
                                    <Button @click="handleDelete(token)" variant="destructive" size="sm" class="ml-2"> Revoke </Button>
                                </TableCell>
                            </TableRow>
                        </TableBody>
                    </Table>
                </CardContent>
            </Card>
        </div>
    </AppLayout>
</template>
