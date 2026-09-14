<?php

declare(strict_types=1);

namespace App\Console\Commands\Statistics;

use App\Actions\Statistics\FinalizeMaintainerReportAction;
use App\Console\Commands\AppCommand;
use App\DTO\MaintainerEntry;
use App\DTO\MaintainerSettings;
use App\Enums\MapAlertType;
use App\Models\Map;
use App\Models\MapAlert;
use App\Services\Discord\DiscordDelivery;
use App\Services\Discord\MaintainerPodiumEmbed;
use App\Services\Statistics\MaintainerPeriod;
use App\Services\Statistics\MaintainerReportReader;
use Carbon\CarbonImmutable;
use Illuminate\Support\Facades\Log;
use Throwable;

use function app;

/**
 * Finalizes every map's maintainer report for the given period (default: the month that
 * just ended) and, for maps with an active MaintainerPodium alert, posts the leaderboard
 * to Discord.
 *
 * No MapAlertDelivery row is reserved for this post -- map_alert_deliveries.map_solarsystem_id
 * is a non-nullable FK and a podium alert has no placement. The per-period
 * discord_posted_at on the report row is the idempotency marker instead; it is a better
 * fit here than a per-placement reservation.
 */
final class PostMaintainerPodiumCommand extends AppCommand
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'app:post-maintainer-podium {--period=} {--map=}';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Finalizes the monthly maintainer report for every map and posts the podium to Discord';

    public function __construct(
        private readonly MaintainerReportReader $reader,
        private readonly MaintainerPodiumEmbed $embedBuilder,
        private readonly DiscordDelivery $delivery,
    ) {
        parent::__construct();
    }

    public function handle(): int
    {
        $period = $this->resolvePeriod();

        $maps = $this->option('map') !== null
            ? Map::query()->where('id', (int) $this->option('map'))->orderBy('id')->get()
            : Map::query()->orderBy('id')->get();

        foreach ($maps as $map) {
            try {
                $this->processMap($map, $period);
            } catch (Throwable $e) {
                Log::error(sprintf('Failed to process map %d for period %s: %s', $map->id, $period->toString(), $e->getMessage()));
            }
        }

        return self::SUCCESS;
    }

    private function processMap(Map $map, MaintainerPeriod $period): void
    {
        $report = app(FinalizeMaintainerReportAction::class)->handle($map, $period);

        if ($report->discord_posted_at !== null) {
            return;
        }

        $alerts = MapAlert::query()
            ->where('map_id', $map->id)
            ->where('type', MapAlertType::MaintainerPodium)
            ->where('is_active', true)
            ->with(['webhook', 'role'])
            ->get();

        if ($alerts->isEmpty()) {
            return;
        }

        $entries = $this->reader->aggregated($map, $period);
        $settings = MaintainerSettings::fromArray($report->payload['settings']);
        $qualifying = $entries
            ->filter(fn (MaintainerEntry $entry): bool => $entry->points >= $settings->minimum_points)
            ->values();

        $embed = $this->embedBuilder->build($alerts->first(), $period, $qualifying, $entries->count());

        foreach ($alerts as $alert) {
            $this->delivery->deliver($alert, $embed);
            $alert->update(['last_fired_at' => CarbonImmutable::now('UTC')]);
        }

        $report->update(['discord_posted_at' => CarbonImmutable::now('UTC')]);
    }

    private function resolvePeriod(): MaintainerPeriod
    {
        $value = $this->option('period');

        return is_string($value) && $value !== ''
            ? MaintainerPeriod::fromString($value)
            : MaintainerPeriod::current()->previous();
    }
}
