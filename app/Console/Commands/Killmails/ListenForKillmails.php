<?php

declare(strict_types=1);

namespace App\Console\Commands\Killmails;

use App\Console\Commands\AppCommand;
use App\Events\Killmails\KillmailReceivedEvent;
use App\Http\Integrations\zKillboard\DTO\RedisQKillmail;
use App\Http\Integrations\zKillboard\zKillboard;
use App\Models\Killmail;
use App\Models\Map;
use App\Models\Solarsystem;
use App\Models\Type;
use Date;
use Exception;
use Illuminate\Container\Attributes\Config;
use Illuminate\Support\Facades\Cache;
use NicolasKion\Esi\Esi;
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
        private readonly Esi $esi,
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
            $redisQKillmail = $this->getNextKillmail();

            if (! $redisQKillmail instanceof RedisQKillmail) {
                $this->info('No killmail received, retrying in 60 seconds...');
                sleep(60);

                continue;
            }

            $result = $this->esi->getKillmail($redisQKillmail->killID, $redisQKillmail->zkb->hash);

            if ($result->failed() || ! $result->data instanceof \NicolasKion\Esi\DTO\Killmail) {
                $this->info(sprintf('Killmail ID %d could not be fetched from ESI, skipping.', $redisQKillmail->killID));

                sleep(self::ZKILL_RATE_LIMIT_SECONDS);

                continue;
            }

            $esi_killmail = $result->data;

            $this->logKillmailDetails($redisQKillmail, $esi_killmail);

            if (! $this->shouldStoreKillmail($esi_killmail)) {
                $this->info(sprintf('Killmail ID %d is older than %d days, skipping.', $redisQKillmail->killID, $this->max_age_days));

                sleep(self::ZKILL_RATE_LIMIT_SECONDS);

                continue;
            }

            $redisQKillmail = $this->processKillmail($redisQKillmail, $esi_killmail);

            $this->notifyMaps($redisQKillmail);

            sleep(self::ZKILL_RATE_LIMIT_SECONDS);
        }

        $this->info('Stopped listening for killmails.');
    }

    private function processKillmail(RedisQKillmail $redis_q_killmail, \NicolasKion\Esi\DTO\Killmail $esi_killmail): Killmail
    {
        return Killmail::query()
            ->updateOrCreate(
                ['id' => $redis_q_killmail->killID],
                [
                    'hash' => $redis_q_killmail->zkb->hash,
                    'solarsystem_id' => $esi_killmail->solar_system_id,
                    'time' => $esi_killmail->killmail_time,
                    'data' => (array) $esi_killmail,
                    'zkb' => (array) $redis_q_killmail->zkb,
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

    private function shouldStoreKillmail(\NicolasKion\Esi\DTO\Killmail $killmail): bool
    {
        $time = Date::parse($killmail->killmail_time);

        return $time->addDays($this->max_age_days)->isFuture();
    }

    private function logKillmailDetails(RedisQKillmail $redisQKillmail, \NicolasKion\Esi\DTO\Killmail $esi_killmail): void
    {
        $solarsystem = Cache::memo()->remember(
            key: sprintf('solarsystem_%d', $esi_killmail->solar_system_id),
            ttl: 3600,
            callback: fn () => Solarsystem::query()->find($esi_killmail->solar_system_id)
        );

        $ship = Cache::memo()->remember(
            key: sprintf('type_%d', $esi_killmail->victim->ship_type_id),
            ttl: 3600,
            callback: fn () => Type::query()->find($esi_killmail->victim->ship_type_id)
        );

        $this->info(sprintf(
            'Received killmail ID %d: A %s was destroyed in %s at %s. https://zkillboard.com/kill/%d/',
            $redisQKillmail->killID,
            $ship->name ?? 'Unknown Ship',
            $solarsystem->name ?? 'Unknown System',
            $esi_killmail->killmail_time,
            $redisQKillmail->killID
        ));
    }
}
