<?php

declare(strict_types=1);

namespace App\Console\Commands\Killmails;

use App\Events\Killmails\KillmailReceivedEvent;
use App\Http\Integrations\zKillboard\DTO\RedisQKillmail;
use App\Http\Integrations\zKillboard\zKillboard;
use App\Models\Killmail;
use App\Models\Map;
use Exception;
use Illuminate\Console\Command;
use Illuminate\Container\Attributes\Config;

use function Laravel\Prompts\error;

final class ListenForKillmails extends Command
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

    private bool $should_keep_running = true;

    /**
     * Execute the console command.
     *
     * @throws Exception
     */
    public function handle(#[Config('services.zkillboard.identifier')] string $identifier, zKillboard $zKillboard): void
    {

        $this->trap([SIGTERM, SIGQUIT], $this->handleStop(...));

        while ($this->should_keep_running) {
            $killmail = $this->getNextKillmail($zKillboard, $identifier);

            if (! $killmail instanceof RedisQKillmail) {
                info('No killmail received, waiting for the next one...');
                sleep(10);

                continue;
            }

            info(sprintf('Received killmail with ID: %d', $killmail->killID));

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
    private function notifyMaps(Killmail $killmail): void
    {
        $maps = Map::query()->whereRelation('mapSolarsystems', 'solarsystem_id', $killmail->solarsystem_id)->get();

        $maps->each(fn (Map $map) => KillmailReceivedEvent::dispatch($map, $killmail));
    }

    private function getNextKillmail(zKillboard $zKillboard, string $identifier): ?RedisQKillmail
    {
        try {
            $killmail = $zKillboard->listenForKill($identifier);
        } catch (Exception $e) {
            error('Error fetching killmail: '.$e->getMessage());

            return null;
        }

        return $killmail;
    }

    private function handleStop(int $signal): void
    {
        switch ($signal) {
            case SIGTERM:
            case SIGQUIT:
                $this->should_keep_running = false;
                break;
        }
    }
}
