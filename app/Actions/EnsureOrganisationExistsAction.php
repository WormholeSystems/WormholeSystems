<?php

declare(strict_types=1);

namespace App\Actions;

use App\Models\Alliance;
use App\Models\Character;
use App\Models\Corporation;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Log;
use NicolasKion\Esi\Esi;
use Throwable;

final readonly class EnsureOrganisationExistsAction
{
    public function __construct(private Esi $esi) {}

    public function ensureCorporationExists(?int $corporation_id): void
    {
        if ($corporation_id === null || $corporation_id === 0) {
            return;
        }

        $resolved = Cache::memo()->remember(
            key: "corporation_resolved_{$corporation_id}",
            ttl: null,
            callback: fn (): bool => Corporation::query()
                ->whereKey($corporation_id)
                ->where(fn ($query) => $query->whereNotNull('name')->orWhereNotNull('unresolvable_at'))
                ->exists(),
        );

        if ($resolved) {
            return;
        }

        $this->refreshCorporation($corporation_id);
    }

    /** A 404 marks the row unresolvable so the periodic name sweep stops retrying it. */
    public function refreshCorporation(int $corporation_id): void
    {
        try {
            $response = $this->esi->getCorporation($corporation_id);

            if ($response->failed()) {
                if ($response->error?->code === 404) {
                    Corporation::query()->whereKey($corporation_id)->update(['unresolvable_at' => now()]);
                }

                Log::info(sprintf('Failed to fetch corporation with ID %d', $corporation_id));

                return;
            }

            $corp_data = $response->data;

            Character::query()->firstOrCreate([
                'id' => $corp_data->ceo_id,
            ]);
            Character::query()->firstOrCreate([
                'id' => $corp_data->creator_id,
            ]);

            if ($corp_data->alliance_id !== null) {
                Alliance::query()->firstOrCreate([
                    'id' => $corp_data->alliance_id,
                ]);
            }

            Corporation::query()->updateOrCreate(
                ['id' => $corporation_id],
                [
                    'name' => $corp_data->name,
                    'ticker' => $corp_data->ticker,
                    'ceo_id' => $corp_data->ceo_id,
                    'alliance_id' => $corp_data->alliance_id,
                    'faction_id' => $corp_data->faction_id,
                    'description' => $corp_data->description,
                    'url' => $corp_data->url,
                    'member_count' => $corp_data->member_count,
                    'shares' => $corp_data->shares,
                    'tax_rate' => $corp_data->tax_rate,
                    'date_founded' => $corp_data->date_founded,
                    'creator_id' => $corp_data->creator_id,
                    'last_updated' => now(),
                    'unresolvable_at' => null,
                ]
            );

            Cache::memo()->put("corporation_resolved_{$corporation_id}", true);
        } catch (Throwable $e) {
            Log::error(sprintf('Error ensuring corporation %d exists: %s', $corporation_id, $e->getMessage()));
        }
    }

    public function ensureAllianceExists(?int $alliance_id): void
    {
        if ($alliance_id === null || $alliance_id === 0) {
            return;
        }

        $resolved = Cache::memo()->remember(
            key: "alliance_resolved_{$alliance_id}",
            ttl: null,
            callback: fn (): bool => Alliance::query()
                ->whereKey($alliance_id)
                ->where(fn ($query) => $query->whereNotNull('name')->orWhereNotNull('unresolvable_at'))
                ->exists(),
        );

        if ($resolved) {
            return;
        }

        $this->refreshAlliance($alliance_id);
    }

    /** A 404 marks the row unresolvable so the periodic name sweep stops retrying it. */
    public function refreshAlliance(int $alliance_id): void
    {
        try {
            $response = $this->esi->getAlliance($alliance_id);

            if ($response->failed()) {
                if ($response->error?->code === 404) {
                    Alliance::query()->whereKey($alliance_id)->update(['unresolvable_at' => now()]);
                }

                Log::info(sprintf('Failed to fetch alliance with ID %d', $alliance_id));

                return;
            }

            $alliance_data = $response->data;

            Character::query()->firstOrCreate([
                'id' => $alliance_data->creator_id,
            ]);
            Corporation::query()->firstOrCreate([
                'id' => $alliance_data->creator_corporation_id,
            ]);

            if ($alliance_data->executor_corporation_id !== null) {
                Corporation::query()->firstOrCreate([
                    'id' => $alliance_data->executor_corporation_id,
                ]);
            }

            Alliance::query()->updateOrCreate(
                ['id' => $alliance_id],
                [
                    'name' => $alliance_data->name,
                    'ticker' => $alliance_data->ticker,
                    'faction_id' => $alliance_data->faction_id,
                    'creator_id' => $alliance_data->creator_id,
                    'creator_corporation_id' => $alliance_data->creator_corporation_id,
                    'date_founded' => $alliance_data->date_founded,
                    'last_updated' => now(),
                    'unresolvable_at' => null,
                ]
            );

            Cache::memo()->put("alliance_resolved_{$alliance_id}", true);
        } catch (Throwable $e) {
            Log::error(sprintf('Error ensuring alliance %d exists: %s', $alliance_id, $e->getMessage()));
        }
    }
}
