<?php

declare(strict_types=1);

namespace App\Jobs\Skyhooks;

use App\Models\RaidableSkyhook;
use Exception;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Foundation\Queue\Queueable;
use Illuminate\Support\Facades\Log;
use NicolasKion\Esi\Esi;

final class GetRaidableSkyhooks implements ShouldQueue
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
     */
    public function handle(Esi $esi): void
    {
        $result = $esi->getRaidableSkyhooks();
        if ($result->failed()) {
            return;
        }

        $skyhooks = collect($result->data);

        foreach ($skyhooks as $skyhook) {
            try {
                RaidableSkyhook::query()->updateOrCreate(
                    ['planet_id' => $skyhook->planet_id],
                    [
                        'solarsystem_id' => $skyhook->solar_system_id,
                        'theft_vulnerability_start' => $skyhook->theft_vulnerability->start,
                        'theft_vulnerability_end' => $skyhook->theft_vulnerability->end,
                    ]
                );
            } catch (Exception $e) {
                Log::info(sprintf('Failed to update raidable skyhook for planet %d: %s', $skyhook->planet_id, $e->getMessage()));
            }
        }

        RaidableSkyhook::query()
            ->whereNotIn('planet_id', $skyhooks->pluck('planet_id'))
            ->delete();
    }
}
