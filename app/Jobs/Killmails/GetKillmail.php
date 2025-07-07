<?php

namespace App\Jobs\Killmails;

use App\Http\Integrations\zKillboard\zKillboard;
use App\Models\Killmail;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Foundation\Queue\Queueable;
use Illuminate\Http\Client\ConnectionException;
use NicolasKion\Esi\Esi;

class GetKillmail implements ShouldQueue
{
    use Queueable;

    /**
     * Create a new job instance.
     */
    public function __construct(public int $killmail_id)
    {
        //
    }

    /**
     * Execute the job.
     *
     * @throws ConnectionException
     */
    public function handle(zKillboard $zKillboard, Esi $esi): void
    {
        $kill = $zKillboard->getKill($this->killmail_id);

        if (empty($kill)) {
            return;
        }

        $esi_killmail = $esi->getKillmail($this->killmail_id, $kill['zkb']['hash']);

        if ($esi_killmail->failed()) {
            // If the ESI request fails, we can log the error or handle it accordingly.
            // For now, we will just return without saving anything.
            return;
        }

        $esi_killmail = $esi_killmail->data;

        Killmail::query()
            ->updateOrCreate(
                ['id' => $kill['killmail_id']],
                [
                    'hash' => $kill['zkb']['hash'],
                    'solarsystem_id' => $esi_killmail->solar_system_id,
                    'time' => $esi_killmail->killmail_time,
                    'data' => (array) $esi_killmail,
                    'zkb' => (array) $kill['zkb'],
                ]
            );
    }
}
