<?php

declare(strict_types=1);

namespace App\Jobs\Characters;

use App\Models\CharacterStatus;
use Illuminate\Bus\Batchable;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Foundation\Queue\Queueable;
use Illuminate\Queue\Middleware\WithoutOverlapping;
use NicolasKion\Esi\Esi;
use Throwable;

use function now;

final class UpdateCharacterOnlineStatus implements ShouldQueue
{
    use Batchable, Queueable;

    /**
     * Create a new job instance.
     */
    public function __construct(public int $character_status_id)
    {
        //
    }

    /**
     * Execute the job.
     */
    public function handle(Esi $esi): void
    {
        $characterStatus = CharacterStatus::query()->find($this->character_status_id);

        if ($characterStatus === null) {
            return;
        }

        try {
            $request = $esi->getOnline($characterStatus->character);
        } catch (Throwable) {
            return;
        }

        if ($request->failed()) {
            return;
        }

        $online_status_changed = $characterStatus->is_online !== $request->data->online;

        if ($online_status_changed) {
            $characterStatus->update([
                'is_online' => $request->data->online,
                'last_online_at' => $request->data->online ? now() : $characterStatus->last_online_at,
                'online_last_checked_at' => now(),
                'event_queued_at' => now(),
            ]);

            return;
        }

        $characterStatus->update([
            'is_online' => $request->data->online,
            'last_online_at' => $request->data->online ? now() : $characterStatus->last_online_at,
            'online_last_checked_at' => now(),
        ]);
    }

    /**
     * Get the middleware the job should pass through.
     *
     * @return array<int, object>
     */
    public function middleware(): array
    {
        return [new WithoutOverlapping((string) $this->character_status_id)->dontRelease()->expireAfter(60)];
    }
}
