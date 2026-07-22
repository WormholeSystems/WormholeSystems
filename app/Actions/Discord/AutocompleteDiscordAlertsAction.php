<?php

declare(strict_types=1);

namespace App\Actions\Discord;

use App\Builders\MapAccessBuilder;
use App\Models\DiscordAccount;
use App\Models\MapAlert;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Support\Facades\Gate;
use Illuminate\Support\Str;

final readonly class AutocompleteDiscordAlertsAction
{
    /** @return array<int, array{name: string, value: string}> */
    public function handle(DiscordAccount $account, string $search): array
    {
        return MapAlert::query()
            ->bot()
            ->where(fn ($query) => $query
                ->where('created_by_user_id', $account->user_id)
                ->orWhereHas('map.mapAccessors', function (Builder $query) use ($account): void {
                    assert($query instanceof MapAccessBuilder);
                    $query->notExpired()->whereIn('accessible_id', $account->user->getAccessibleIds());
                }))
            ->with(['map', 'targetSolarsystem:id,name'])
            ->when($search !== '', fn ($query) => $query->where(function ($query) use ($search): void {
                $query->where('id', 'like', '%'.$search.'%')
                    ->orWhereHas('map', fn ($query) => $query->where('name', 'like', '%'.$search.'%'))
                    ->orWhereHas('targetSolarsystem', fn ($query) => $query->where('name', 'like', '%'.$search.'%'));
            }))
            ->latest()
            ->get()
            ->filter(fn (MapAlert $alert): bool => Gate::forUser($account->user)->allows('update', $alert))
            ->take(25)
            ->map(fn (MapAlert $alert): array => [
                'name' => Str::limit(sprintf('%s: %s on %s', $alert->id, $alert->targetSolarsystem->name, $alert->map->name), 100, ''),
                'value' => (string) $alert->id,
            ])
            ->values()
            ->all();
    }
}
