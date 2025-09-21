<?php

declare(strict_types=1);

namespace App\Console\Commands\Killmails;

use App\Console\Commands\AppCommand;
use App\Events\Killmails\KillmailReceivedEvent;
use App\Http\Integrations\zKillboard\DTO\RedisQKillmail;
use App\Http\Integrations\zKillboard\zKillboard;
use App\Models\Killmail;
use App\Models\Map;
use Date;
use Exception;
use Illuminate\Container\Attributes\Config;
use Throwable;

use function sleep;
use function sprintf;

final class ListenForKillmails extends AppCommand
{
    private const int ZKILL_RATE_LIMIT_SECONDS = 1;

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

    public function __construct(
        private readonly zKillboard $zKillboard,
        #[Config('services.zkillboard.identifier')] private readonly string $identifier,
        #[Config('services.zkillboard.max_age_days')] private readonly int $max_age_days,
    ) {
        parent::__construct();
    }

    /**
     * Execute the console command.
     *
     * @throws Exception
     */
    public function handle(): void
    {

        $this->trap([SIGTERM, SIGQUIT], $this->handleStop(...));

        while ($this->should_keep_running) {
            $killmail = $this->getNextKillmail();

            if (! $killmail instanceof RedisQKillmail) {
                $this->info('No killmail received, retrying in 60 seconds...');
                sleep(60);

                continue;
            }

            $this->info(sprintf('Received killmail ID %d in system %d at %s', $killmail->killID, $killmail->killmail->solar_system_id, $killmail->killmail->killmail_time));

            if (! $this->shouldStoreKillmail($killmail)) {
                $this->info(sprintf('Killmail ID %d is older than %d days, skipping.', $killmail->killID, $this->max_age_days));

                sleep(self::ZKILL_RATE_LIMIT_SECONDS);

                continue;
            }

            $killmail = $this->processKillmail($killmail);

            $this->notifyMaps($killmail);

            sleep(self::ZKILL_RATE_LIMIT_SECONDS);
        }

        $this->info('Stopped listening for killmails.');
    }

    private function processKillmail(RedisQKillmail $killmail): Killmail
    {
        return Killmail::query()
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
    }

    /**
     * Notify maps about the new killmail.
     */
    private function notifyMaps(Killmail $killmail): void
    {
        $maps = Map::query()->whereRelation('mapSolarsystems', 'solarsystem_id', $killmail->solarsystem_id)->get();

        $maps->each(fn (Map $map) => KillmailReceivedEvent::dispatch($map));
    }

    private function getNextKillmail(): ?RedisQKillmail
    {
        try {
            $killmail = $this->zKillboard->listenForKill($this->identifier);
        } catch (Throwable $e) {
            $this->error(sprintf('Error while listening for killmail: %s', $e->getMessage()));

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

    private function shouldStoreKillmail(RedisQKillmail $killmail): bool
    {
        $time = Date::parse($killmail->killmail->killmail_time);

        return $time->addDays($this->max_age_days)->isFuture();
    }
}
