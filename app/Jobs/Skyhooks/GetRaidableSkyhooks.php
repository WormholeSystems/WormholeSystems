<?php

declare(strict_types=1);

namespace App\Jobs\Skyhooks;

use App\Models\RaidableSkyhook;
use App\Models\Solarsystem;
use Carbon\CarbonImmutable;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Foundation\Queue\Queueable;
use Illuminate\Support\Facades\Log;
use NicolasKion\Esi\DTO\RaidableSkyhook as RaidableSkyhookData;
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

        $known_solarsystem_ids = Solarsystem::query()
            ->whereIn('id', $skyhooks->pluck('solar_system_id')->unique())
            ->pluck('id')
            ->flip();

        [$rows, $unknown] = $skyhooks->partition(
            fn (RaidableSkyhookData $skyhook): bool => $known_solarsystem_ids->has($skyhook->solar_system_id)
        );

        $unknown->each(fn (RaidableSkyhookData $skyhook) => Log::info(sprintf(
            'Skipping raidable skyhook for planet %d in unknown solarsystem %d',
            $skyhook->planet_id,
            $skyhook->solar_system_id,
        )));

        RaidableSkyhook::query()->upsert(
            $rows->map(fn (RaidableSkyhookData $skyhook): array => [
                'planet_id' => $skyhook->planet_id,
                'solarsystem_id' => $skyhook->solar_system_id,
                'theft_vulnerability_start' => CarbonImmutable::parse($skyhook->theft_vulnerability->start)->toDateTimeString(),
                'theft_vulnerability_end' => CarbonImmutable::parse($skyhook->theft_vulnerability->end)->toDateTimeString(),
            ])->values()->all(),
            ['planet_id'],
            ['solarsystem_id', 'theft_vulnerability_start', 'theft_vulnerability_end', 'updated_at'],
        );

        RaidableSkyhook::query()
            ->whereNotIn('planet_id', $skyhooks->pluck('planet_id'))
            ->delete();
    }
}
