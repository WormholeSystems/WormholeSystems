<?php

declare(strict_types=1);

namespace App\Services\Discord;

use App\Models\Solarsystem;

trait ResolvesSolarsystems
{
    /**
     * @param  array<int, int|null>  $ids
     * @return array<int, array{name: string, security: float}>
     */
    private function resolveSystems(array $ids): array
    {
        return Solarsystem::query()
            ->whereIn('id', array_unique(array_filter($ids)))
            ->get(['id', 'name', 'security'])
            ->keyBy('id')
            ->map(fn (Solarsystem $system): array => ['name' => $system->name, 'security' => (float) $system->security])
            ->all();
    }
}
