<?php

declare(strict_types=1);

namespace App\Console\Commands\Killmails;

use App\Actions\EnsureOrganisationExistsAction;
use App\Console\Commands\AppCommand;
use App\Events\Killmails\KillmailReceivedEvent;
use App\Http\Integrations\zKillboard\DTO\R2Z2Killmail;
use App\Http\Integrations\zKillboard\zKillboard;
use App\Models\Killmail;
use App\Models\Map;
use App\Models\Solarsystem;
use App\Models\Type;
use Date;
use Illuminate\Container\Attributes\Config;
use Illuminate\Support\Facades\Cache;
use Throwable;

use function sleep;
use function sprintf;
use function usleep;

final class ListenForKillmails extends AppCommand
{
    public const string RESTART_CACHE_KEY = 'zkillboard:r2z2:restart';

    private const string CACHE_KEY = 'zkillboard:r2z2:last_sequence';

    private const int POLL_DELAY_MS = 100;

    private const int CATCHUP_SLEEP_SECONDS = 6;

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
    protected $description = 'Listen to zKillboard R2Z2 stream for killmails and process them.';

    private bool $should_keep_running = true;

    public function __construct(
        private readonly zKillboard $zKillboard,
        private readonly EnsureOrganisationExistsAction $ensureOrganisationExistsAction,
        #[Config('services.zkillboard.max_age_days')] private readonly int $max_age_days,
    ) {
        parent::__construct();
    }

    public function handle(): int
    {
        $this->trap([SIGTERM, SIGQUIT], $this->handleStop(...));

        $restartSignal = Cache::get(self::RESTART_CACHE_KEY);

        $sequence = $this->getStartingSequence();
        $this->info(sprintf('Starting R2Z2 polling from sequence %d', $sequence));

        while ($this->should_keep_running) {
            if ($this->shouldRestart($restartSignal)) {
                $this->info('Restart signal detected, exiting.');

                return self::SUCCESS;
            }
            try {
                $killmail = $this->zKillboard->getKillmailBySequence($sequence);
            } catch (Throwable $e) {
                $this->error(sprintf('Error fetching sequence %d: %s', $sequence, $e->getMessage()));
                sleep(self::CATCHUP_SLEEP_SECONDS);

                continue;
            }

            if (! $killmail instanceof R2Z2Killmail) {
                // 404 or empty — we've caught up, wait and retry same sequence
                sleep(self::CATCHUP_SLEEP_SECONDS);

                continue;
            }

            $this->processKillmail($killmail);

            // Advance sequence and persist
            $sequence = $killmail->sequence_id + 1;
            Cache::put(self::CACHE_KEY, $sequence);

            // Brief delay between active polls to respect rate limits
            usleep(self::POLL_DELAY_MS * 1000);
        }

        $this->info('Stopped listening for killmails.');

        return self::SUCCESS;
    }

    private function shouldRestart(mixed $restartSignal): bool
    {
        return Cache::get(self::RESTART_CACHE_KEY) !== $restartSignal;
    }

    private function getStartingSequence(): int
    {
        $cached = Cache::get(self::CACHE_KEY);

        if ($cached) {
            return (int) $cached;
        }

        // No cached sequence — start from the current latest
        try {
            $latest = $this->zKillboard->getLatestSequence();
            $this->info(sprintf('No cached sequence found, starting from latest: %d', $latest));

            return $latest;
        } catch (Throwable $e) {
            $this->error(sprintf('Could not fetch latest sequence: %s', $e->getMessage()));

            return 0;
        }
    }

    private function processKillmail(R2Z2Killmail $killmail): void
    {
        $this->logKillmailDetails($killmail);

        if (! $this->shouldStoreKillmail($killmail)) {
            $this->info(sprintf('Killmail ID %d is older than %d days, skipping.', $killmail->killmail_id, $this->max_age_days));

            return;
        }

        $stored = Killmail::query()->updateOrCreate(
            ['id' => $killmail->killmail_id],
            [
                'hash' => $killmail->hash,
                'solarsystem_id' => $killmail->getSolarSystemId(),
                'time' => $killmail->getKillmailTime(),
                'data' => $killmail->esi,
                'zkb' => (array) $killmail->zkb,
            ]
        );

        $this->ensureOrganisationExistsAction->ensureCorporationExists($killmail->getVictimCorporationId());
        $this->ensureOrganisationExistsAction->ensureAllianceExists($killmail->getVictimAllianceId());

        $this->notifyMaps($stored);
    }

    private function notifyMaps(Killmail $killmail): void
    {
        $maps = Map::query()->whereRelation('mapSolarsystems', 'solarsystem_id', $killmail->solarsystem_id)->get();

        $maps->each(fn (Map $map) => KillmailReceivedEvent::dispatch($map));
    }

    private function shouldStoreKillmail(R2Z2Killmail $killmail): bool
    {
        $time = $killmail->getKillmailTime();

        if (! $time) {
            return false;
        }

        return Date::parse($time)->addDays($this->max_age_days)->isFuture();
    }

    private function logKillmailDetails(R2Z2Killmail $killmail): void
    {
        $solarsystem = Cache::memo()->remember(
            key: sprintf('solarsystem_%d', $killmail->getSolarSystemId()),
            ttl: 3600,
            callback: fn () => Solarsystem::query()->find($killmail->getSolarSystemId())
        );

        $ship = Cache::memo()->remember(
            key: sprintf('type_%d', $killmail->getVictimShipTypeId()),
            ttl: 3600,
            callback: fn () => Type::query()->find($killmail->getVictimShipTypeId())
        );

        $this->info(sprintf(
            '[seq:%d] Killmail %d: %s destroyed in %s at %s. https://zkillboard.com/kill/%d/',
            $killmail->sequence_id,
            $killmail->killmail_id,
            $ship->name ?? 'Unknown Ship',
            $solarsystem->name ?? 'Unknown System',
            $killmail->getKillmailTime() ?? 'unknown time',
            $killmail->killmail_id
        ));
    }

    private function handleStop(int $signal): void
    {
        match ($signal) {
            SIGTERM, SIGQUIT => $this->should_keep_running = false,
            default => null,
        };
    }
}
