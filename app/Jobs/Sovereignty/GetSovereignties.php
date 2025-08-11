<?php

declare(strict_types=1);

namespace App\Jobs\Sovereignty;

use App\Models\Alliance;
use App\Models\Corporation;
use App\Models\Faction;
use App\Models\Sovereignty;
use Exception;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Foundation\Queue\Queueable;
use Illuminate\Http\Client\ConnectionException;
use Illuminate\Support\Facades\Log;
use NicolasKion\Esi\DTO\Name;
use NicolasKion\Esi\Enums\NameCategory;
use NicolasKion\Esi\Esi;

final class GetSovereignties implements ShouldQueue
{
    use Queueable;

    /**
     * Create a new job instance.
     */
    public function __construct()
    {
        //
    }

    /**
     * Execute the job.
     *
     * @throws ConnectionException
     */
    public function handle(Esi $esi): void
    {
        $result = $esi->getSovereignty();
        if ($result->failed()) {
            return;
        }

        $sovereignties = collect($result->data);

        $alliance_ids = $sovereignties->pluck('alliance_id')->filter()->unique();
        $faction_ids = $sovereignties->pluck('faction_id')->filter()->unique();
        $corporation_ids = $sovereignties->pluck('corporation_id')->filter()->unique();

        // Check what IDs do not exist in the database
        $missing_alliance_ids = $alliance_ids->filter(fn (int $id): bool => ! Alliance::query()->whereId($id)->exists());
        $missing_faction_ids = $faction_ids->filter(fn (int $id): bool => ! Faction::query()->whereId($id)->exists());
        $missing_corporation_ids = $corporation_ids->filter(fn (int $id): bool => ! Corporation::query()->whereId($id)->exists());

        $this->getNamesForMissingEntities($esi, $missing_corporation_ids->merge($missing_alliance_ids)->merge($missing_faction_ids));

        foreach ($sovereignties as $sovereignty) {
            try {
                Sovereignty::query()
                    ->updateOrCreate(
                        ['solarsystem_id' => $sovereignty->system_id],
                        [
                            'alliance_id' => $sovereignty->alliance_id,
                            'faction_id' => $sovereignty->faction_id,
                            'corporation_id' => $sovereignty->corporation_id,
                        ]
                    );
            } catch (Exception $e) {
                Log::info(sprintf('Failed to update sovereignty for system %d: %s', $sovereignty->system_id, $e->getMessage()));
            }
        }
    }

    /**
     * @throws ConnectionException
     */
    private function getNamesForMissingEntities(Esi $esi, \Illuminate\Support\Collection $missing_ids): void
    {
        if ($missing_ids->isEmpty()) {
            return;
        }

        $result = $esi->getNames($missing_ids->toArray());
        if ($result->failed()) {
            return;
        }
        $names = collect($result->data);

        $names->each(static fn (Name $name) => match ($name->category) {
            NameCategory::Alliance => Alliance::query()->updateOrCreate(['id' => $name->id], ['name' => $name->name]),
            NameCategory::Corporation => Corporation::query()->updateOrCreate(['id' => $name->id], ['name' => $name->name]),
            NameCategory::Faction => Faction::query()->updateOrCreate(['id' => $name->id], ['name' => $name->name]),
            default => null,
        });
    }
}
