<?php

namespace App\Console\Commands\Killmails;

use App\Events\Killmails\KillmailReceivedEvent;
use App\Http\Integrations\zKillboard\zKillboard;
use App\Models\Killmail;
use App\Models\Map;
use Exception;
use Illuminate\Console\Command;
use Illuminate\Container\Attributes\Config;

class ListenForKillmails extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'app:listen-for-killmails';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Listen to zKillboard for killmails and process them.';

    protected bool $should_keep_running = true;

    /**
     * Execute the console command.
     *
     * @throws Exception
     */
    public function handle(#[Config('services.zkillboard.identifier')] string $identifier, zKillboard $zKillboard): void
    {

        $this->trap([SIGTERM, SIGQUIT], fn (): false => $this->should_keep_running = false);

        while ($this->should_keep_running) {
            $killmail = $zKillboard->listenForKill($identifier);

            if (! $killmail instanceof \App\Http\Integrations\zKillboard\DTO\RedisQKillmail) {
                info('No killmail received, waiting for the next one...');
                sleep(10);

                continue;
            }

            \Laravel\Prompts\info(sprintf('Received killmail with ID: %d', $killmail->killID));

            $killmail = Killmail::query()
                ->updateOrCreate(
                    ['id' => $killmail->killID],
                    [
                        'hash' => $killmail->zkb->hash,
                        'solarsystem_id' => $killmail->killmail->solar_system_id,
                        'time' => $killmail->killmail->killmail_time,
                        'data' => (array) $killmail->killmail,
                        'zkb' => (array) $killmail->zkb,
                    ]
                );

            $this->notifyMaps($killmail);
        }
    }

    /**
     * Notify maps about the new killmail.
     */
    protected function notifyMaps(Killmail $killmail): void
    {
        $maps = Map::query()->whereRelation('mapSolarsystems', 'solarsystem_id', $killmail->solarsystem_id)->get();

        $maps->each(fn (Map $map) => KillmailReceivedEvent::dispatch($map, $killmail));
    }
}
