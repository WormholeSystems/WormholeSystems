<?php

namespace App\Jobs;

use App\Models\Alliance;
use App\Models\Character;
use App\Models\Corporation;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Foundation\Queue\Queueable;
use Illuminate\Http\Client\ConnectionException;
use Illuminate\Support\Facades\Log;
use NicolasKion\Esi\DTO\CharacterAffiliation;
use NicolasKion\Esi\Esi;

class UpdateAffiliations implements ShouldQueue
{
    use Queueable;

    /**
     * Create a new job instance.
     */
    public function __construct(public CharacterAffiliation $affiliation)
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
        $this->updateCorporation($esi);
        $this->updateAlliance($esi);
    }

    /**
     * @throws ConnectionException
     */
    private function updateCorporation(Esi $esi): void
    {
        $corporation_doesnt_need_update = Corporation::query()
            ->where('id', $this->affiliation->corporation_id)
            ->where('last_updated', '>=', now()->subDays(1))
            ->exists();

        if ($corporation_doesnt_need_update) {
            Log::info(sprintf('Corporation with ID %d is up-to-date, skipping update.', $this->affiliation->corporation_id));

            return;
        }

        $corporation_response = $esi->getCorporation($this->affiliation->corporation_id);

        if ($corporation_response->failed()) {
            Log::info(sprintf('Failed to fetch corporation with ID %d', $this->affiliation->corporation_id));

            return;
        }

        $corp_data = $corporation_response->data;

        // Ensure related models exist before updating or creating
        Character::query()->firstOrCreate([
            'id' => $corp_data->ceo_id,
        ]);
        Character::query()->firstOrCreate([
            'id' => $corp_data->creator_id,
        ]);

        if ($corp_data->alliance_id !== null) {
            Corporation::query()->firstOrCreate([
                'id' => $corp_data->alliance_id,
            ]);
        }

        Corporation::query()->updateOrCreate(
            ['id' => $this->affiliation->corporation_id],
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
                'home_station_id' => $corp_data->home_station_id,
                'date_founded' => $corp_data->date_founded,
                'creator_id' => $corp_data->creator_id,
                'last_updated' => now(),
            ]
        );

        Log::info(sprintf('Updated corporation with ID %d', $this->affiliation->corporation_id));
    }

    /**
     * @throws ConnectionException
     */
    private function updateAlliance(Esi $esi): void
    {
        if ($this->affiliation->alliance_id === null || $this->affiliation->alliance_id === 0) {
            return;
        }

        $alliance_doesnt_need_update = Corporation::query()
            ->where('id', $this->affiliation->alliance_id)
            ->where('last_updated', '>=', now()->subDays(1))
            ->exists();

        if ($alliance_doesnt_need_update) {
            Log::info(sprintf('Alliance with ID %d is up-to-date, skipping update.', $this->affiliation->alliance_id));

            return;
        }

        $alliance_response = $esi->getAlliance($this->affiliation->alliance_id);

        if ($alliance_response->failed()) {
            Log::info(sprintf('Failed to fetch alliance with ID %d', $this->affiliation->alliance_id));

            return;
        }

        $alliance_data = $alliance_response->data;

        // Ensure related models exist before updating or creating
        Character::query()->firstOrCreate([
            'id' => $alliance_data->creator_id,
        ]);
        Corporation::query()->firstOrCreate([
            'id' => $alliance_data->executor_corporation_id,
        ]);

        Alliance::query()->updateOrCreate(
            ['id' => $this->affiliation->alliance_id],
            [
                'name' => $alliance_data->name,
                'ticker' => $alliance_data->ticker,
                'faction_id' => $alliance_data->faction_id,
                'creator_id' => $alliance_data->creator_id,
                'date_founded' => $alliance_data->date_founded,
                'last_updated' => now(),
            ]
        );

        Log::info(sprintf('Updated alliance with ID %d', $this->affiliation->alliance_id));
    }
}
