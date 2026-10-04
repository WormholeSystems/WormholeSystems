<?php

declare(strict_types=1);

namespace App\Services\Discord;

use App\Builders\MapAccessBuilder;
use App\Models\DiscordAccount;
use App\Models\Map;
use App\Models\Solarsystem;
use Illuminate\Support\Str;

final class DiscordAutocomplete
{
    /** @return array<int, array{name: string, value: string}> */
    public function maps(DiscordAccount $account, string $search): array
    {
        return Map::query()
            ->whereHas('mapAccessors', function (MapAccessBuilder $query) use ($account): void {
                $query->notExpired()->whereIn('accessible_id', $account->user->getAccessibleIds());
            })
            ->when($search !== '', fn ($query) => $query->where('name', 'like', '%'.$search.'%'))
            ->orderBy('name')
            ->limit(25)
            ->get(['id', 'name'])
            ->map(fn (Map $map): array => ['name' => $map->name, 'value' => (string) $map->id])
            ->all();
    }

    /** @return array<int, array{name: string, value: string}> */
    public function solarsystems(string $search): array
    {
        return Solarsystem::query()
            ->when($search !== '', fn ($query) => $query->search(Str::of($search)))
            ->orderBy('name')
            ->limit(25)
            ->get(['id', 'name'])
            ->map(fn (Solarsystem $system): array => ['name' => $system->name, 'value' => (string) $system->id])
            ->all();
    }
}
