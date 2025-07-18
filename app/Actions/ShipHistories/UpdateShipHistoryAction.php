<?php

declare(strict_types=1);

namespace App\Actions\ShipHistories;

use App\Models\ShipHistory;
use Illuminate\Support\Facades\DB;
use Throwable;

final readonly class UpdateShipHistoryAction
{
    /**
     * Execute the action.
     *
     * @throws Throwable
     */
    public function handle(int $character_id, int $ship_id, int $ship_type_id, ?string $name = ''): ShipHistory
    {
        return DB::transaction(function () use ($ship_id, $ship_type_id, $name, $character_id) {
            $history = $this->upsertShipHistory(
                $ship_id,
                $ship_type_id,
                $character_id,
                $name
            );

            return $history;
        });
    }

    private function upsertShipHistory(
        int $ship_id,
        int $ship_type_id,
        int $character_id,
        ?string $name = ''
    ): ShipHistory {
        $history = ShipHistory::query()->latest()->firstOrCreate(
            ['ship_id' => $ship_id],
            [
                'ship_type_id' => $ship_type_id,
                'character_id' => $character_id,
                'name' => $name,
            ]
        );

        if ($history->wasRecentlyCreated) {
            return $history;
        }

        $history->touch('updated_at');

        if ($history->character_id !== $character_id) {
            return ShipHistory::query()
                ->create([
                    'ship_id' => $ship_id,
                    'ship_type_id' => $ship_type_id,
                    'character_id' => $character_id,
                    'name' => $name,
                ]);
        }

        $history->update([
            'name' => $name,
        ]);

        return $history;
    }
}
