<?php

namespace App\Console\Commands\Killmails;

use App\Enums\MapSolarsystemStatus;
use App\Events\MapSolarsystems\MapSolarsystemsUpdatedEvent;
use App\Models\Alliance;
use App\Models\Corporation;
use App\Models\MapSolarsystem;
use App\Models\WormholeSystem;
use Illuminate\Console\Command;
use Illuminate\Http\Client\ConnectionException;
use Illuminate\Support\Collection;
use Illuminate\Support\Facades\Cache;
use NicolasKion\Esi\DTO\Name;
use NicolasKion\Esi\Enums\NameCategory;
use NicolasKion\Esi\Esi;
use Psr\SimpleCache\InvalidArgumentException;

use function Laravel\Prompts\error;
use function Laravel\Prompts\progress;

class AnalyzeWormholeSystems extends Command
{
    private int $mapId;

    private int $daysAgo;

    private int $daysActive;

    private int $top;

    private int $activeThreshold;

    private int $hostileThreshold;

    public function __construct(private readonly Esi $esi)
    {
        parent::__construct();
    }

    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'app:analyze-wormhole-systems
        {map : The map ID to analyze}
        {--days-ago=90 : Number of days to look back for killmails}
        {--days-active=5 : Minimum number of days with kills to consider a group active}
        {--top=10 : Number of top groups to display}
        {--active-threshold=15 : Minimum number of kills to consider a system active}
        {--hostile-threshold=50 : Minimum number of kills to consider a system hostile}';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Analyze wormhole systems based on killmails';

    /**
     * Execute the console command.
     */
    public function handle(): int
    {
        $this->initializeOptions();

        progress('Analyzing wormhole systems', WormholeSystem::all(), fn (WormholeSystem $wormholeSystem) => $this->analyzeWormholeSystem($wormholeSystem));

        MapSolarsystemsUpdatedEvent::dispatch($this->mapId);

        return self::SUCCESS;
    }

    private function initializeOptions(): void
    {

        $this->mapId = (int) $this->argument('map');
        $this->daysAgo = (int) $this->option('days-ago');
        $this->daysActive = (int) $this->option('days-active');
        $this->top = (int) $this->option('top');
        $this->activeThreshold = (int) $this->option('active-threshold');
        $this->hostileThreshold = (int) $this->option('hostile-threshold');
    }

    private function analyzeWormholeSystem(WormholeSystem $wormholeSystem): void
    {
        $killmails = $this->getKillmails($wormholeSystem);

        $organizationStats = $this->calculateOrganizationStats($killmails);
        $topOrganizations = $this->getTopOrganizations($organizationStats);

        $this->ensureNamesExist($topOrganizations->keys());

        $systemStatus = $this->determineSystemStatus($topOrganizations);
        $notesContent = $this->generateNotesContent($topOrganizations);

        $this->updateMapSolarsystem($wormholeSystem, $systemStatus, $notesContent);
    }

    private function getKillmails(WormholeSystem $wormholeSystem): Collection
    {
        return $wormholeSystem->solarsystem
            ->killmails()
            ->where('time', '>=', now()->subDays($this->daysAgo))
            ->get();
    }

    private function calculateOrganizationStats(Collection $killmails): Collection
    {
        $organizationStats = [];

        foreach ($killmails as $killmail) {
            $killDate = $killmail->time->format('Y-m-d');
            $participatingOrganizations = $this->getParticipatingOrganizations($killmail);

            // Count each organization only once per killmail to avoid double counting
            foreach ($participatingOrganizations as $organizationId => $organizationType) {
                $this->addOrganizationActivity($organizationStats, $organizationId, $organizationType, $killDate);
            }
        }

        return collect($organizationStats)
            ->filter(fn (array $stats) => count($stats['active_days']) >= $this->daysActive);
    }

    private function getParticipatingOrganizations(object $killmail): array
    {
        $organizations = [];

        // Add victim's organization
        [$type, $id] = $this->getOrganizationFromEntity($killmail->data->victim);
        if ($id !== 0) {
            $organizations[$id] = $type;
        }

        // Add unique attacker organizations (deduplicates automatically using array keys)
        foreach ($killmail->data->attackers as $attacker) {
            [$type, $id] = $this->getOrganizationFromEntity($attacker);
            if ($id !== 0) {
                $organizations[$id] = $type;
            }
        }

        return $organizations;
    }

    private function addOrganizationActivity(array &$organizationStats, int $organizationId, string $type, string $date): void
    {
        if (! isset($organizationStats[$organizationId])) {
            $organizationStats[$organizationId] = [
                'type' => $type,
                'kill_count' => 0,
                'active_days' => [],
            ];
        }

        $organizationStats[$organizationId]['kill_count']++;
        $organizationStats[$organizationId]['active_days'][$date] = true;
    }

    private function getTopOrganizations(Collection $organizationStats): Collection
    {
        return $organizationStats
            ->sortByDesc('kill_count')
            ->take($this->top);
    }

    private function determineSystemStatus(Collection $topOrganizations): MapSolarsystemStatus
    {
        $totalKills = $topOrganizations->sum('kill_count');

        return match (true) {
            $totalKills >= $this->hostileThreshold => MapSolarsystemStatus::Hostile,
            $totalKills >= $this->activeThreshold => MapSolarsystemStatus::Active,
            default => MapSolarsystemStatus::Unknown
        };
    }

    private function generateNotesContent(Collection $topOrganizations): string
    {
        $start = '<!-- killmails:start -->';
        $end = '<!-- killmails:end -->';

        if ($topOrganizations->isEmpty()) {
            $markdown = 'We could not find any groups that meet the criteria.';
        } else {
            $markdown = $topOrganizations
                ->map(fn (array $stats, int $id) => $this->getEntityDetails($stats, $id))
                ->implode("\n");
        }

        return sprintf(
            "%s\nTop %d groups that were active for at least %d days within the last %d days:\n\n%s\n\n%s\n",
            $start,
            $this->top,
            $this->daysActive,
            $this->daysAgo,
            $markdown,
            $end
        );
    }

    private function updateMapSolarsystem(WormholeSystem $wormholeSystem, MapSolarsystemStatus $status, string $newNotesContent): void
    {
        $mapSolarsystem = MapSolarsystem::query()
            ->where('solarsystem_id', $wormholeSystem->solarsystem->id)
            ->where('map_id', $this->mapId)
            ->firstOrNew();

        $updatedNotes = $this->mergeNotesContent($mapSolarsystem->notes ?? '', $newNotesContent);

        $mapSolarsystem->fill([
            'solarsystem_id' => $wormholeSystem->solarsystem->id,
            'map_id' => $this->mapId,
            'notes' => $updatedNotes,
            'status' => $status,
        ]);

        $mapSolarsystem->save();
    }

    private function mergeNotesContent(string $existingNotes, string $newNotesContent): string
    {
        $hasExistingBlock = preg_match('/<!-- killmails:start -->.*<!-- killmails:end -->/s', $existingNotes);

        if ($hasExistingBlock) {
            return preg_replace(
                '/<!-- killmails:start -->.*<!-- killmails:end -->/s',
                $newNotesContent,
                $existingNotes
            );
        }

        return $existingNotes.$newNotesContent;
    }

    private function ensureNamesExist(Collection $organizationIds): void
    {
        $organizationIds->each(/**
         * @throws InvalidArgumentException
         * @throws ConnectionException
         */ fn (int $id) => $this->ensureNameExists($id));
    }

    /**
     * @throws InvalidArgumentException
     * @throws ConnectionException
     */
    private function ensureNameExists(int $id): void
    {
        if (Cache::memo()->get($id)) {
            return;
        }

        if (Corporation::query()->whereId($id)->whereNotNull('name')->exists()) {
            Cache::memo()->put($id, true);

            return;
        }

        if (Alliance::query()->whereId($id)->whereNotNull('name')->exists()) {
            Cache::memo()->put($id, true);

            return;
        }

        $request = $this->esi->getNames([$id]);

        if ($request->failed()) {
            error(sprintf('Failed to get name for ID %d', $id));

            return;
        }

        /** @var Name $data */
        [$data] = $request->data;

        if ($data->category === NameCategory::Corporation) {
            Corporation::query()->updateOrCreate(
                ['id' => $data->id],
                ['name' => $data->name]
            );
            Cache::memo()->put($data->id, true);

            return;
        }

        if ($data->category === NameCategory::Alliance) {
            Alliance::query()->updateOrCreate(
                ['id' => $data->id],
                ['name' => $data->name]
            );
            Cache::memo()->put($data->id, true);

            return;
        }

        error(sprintf('Unknown name category for ID %d: %s', $id, $data->category->value));
    }

    private function getOrganizationFromEntity(object $entity): array
    {
        if (isset($entity->alliance_id) && $entity->alliance_id > 0) {
            return ['alliance', $entity->alliance_id];
        }

        if (isset($entity->corporation_id) && $entity->corporation_id > 0) {
            return ['corporation', $entity->corporation_id];
        }

        return ['unknown', 0];
    }

    private function getEntityDetails(array $organizationStats, int $id): string
    {
        $alliance = Alliance::query()->find($id);
        if ($alliance) {
            return $this->getAllianceDetails($alliance, $organizationStats['kill_count']);
        }

        $corporation = Corporation::query()->find($id);
        if ($corporation) {
            return $this->getCorporationDetails($corporation, $organizationStats['kill_count']);
        }

        return sprintf('Unknown entity with ID %d and %d kills', $id, $organizationStats['kill_count']);
    }

    private function getAllianceDetails(Alliance $alliance, int $kills): string
    {
        return sprintf(
            '- [%s](https://zkillboard.com/alliance/%d/) - %d kills',
            $alliance->name,
            $alliance->id,
            $kills
        );
    }

    private function getCorporationDetails(Corporation $corporation, int $kills): string
    {
        return sprintf(
            '- [%s](https://zkillboard.com/corporation/%d/) - %d kills',
            $corporation->name,
            $corporation->id,
            $kills
        );
    }
}
