<?php

declare(strict_types=1);

namespace App\Console\Commands\Killmails;

use App\Collections\OrganisationStatsCollection;
use App\Console\Commands\AppCommand;
use App\DTO\AnalysisOptions;
use App\DTO\SystemAnalysisResult;
use App\Events\MapSolarsystems\MapSolarsystemsUpdatedEvent;
use App\Models\Alliance;
use App\Models\Corporation;
use App\Models\MapSolarsystem;
use App\Models\WormholeSystem;
use Illuminate\Http\Client\ConnectionException;
use Illuminate\Support\Collection;
use Illuminate\Support\Facades\Cache;
use NicolasKion\Esi\DTO\Name;
use NicolasKion\Esi\Enums\NameCategory;
use NicolasKion\Esi\Esi;
use Psr\SimpleCache\InvalidArgumentException;

use function Laravel\Prompts\progress;

final class AnalyzeWormholeSystems extends AppCommand
{
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

    private AnalysisOptions $options;

    public function __construct(private readonly Esi $esi)
    {
        parent::__construct();
    }

    /**
     * Execute the console command.
     */
    public function handle(): int
    {
        $this->options = AnalysisOptions::fromCommand(
            $this->arguments(),
            $this->options()
        );

        progress('Analyzing wormhole systems', WormholeSystem::all(), $this->analyzeWormholeSystem(...));

        MapSolarsystemsUpdatedEvent::dispatch($this->options->mapId);

        $this->info('Finished analyzing wormhole systems');

        return self::SUCCESS;
    }

    private function analyzeWormholeSystem(WormholeSystem $wormholeSystem): void
    {
        $killmails = $this->getKillmails($wormholeSystem);

        $organisationStats = $this->calculateOrganisationStats($killmails);
        $topOrganisations = $this->getTopOrganisations($organisationStats);

        $this->ensureNamesExist($topOrganisations->getOrganisationIds());

        $analysisResult = SystemAnalysisResult::create($topOrganisations, $this->options);

        $this->updateMapSolarsystem($wormholeSystem, $analysisResult);
    }

    private function getKillmails(WormholeSystem $wormholeSystem): Collection
    {
        return $wormholeSystem->solarsystem
            ->killmails()
            ->where('time', '>=', now()->subDays($this->options->daysAgo))
            ->get();
    }

    private function calculateOrganisationStats(Collection $killmails): OrganisationStatsCollection
    {
        $organisationStats = new OrganisationStatsCollection();

        foreach ($killmails as $killmail) {
            $killDate = $killmail->time->format('Y-m-d');
            $participatingOrganisations = $this->getParticipatingOrganisations($killmail);

            // Count each organisation only once per killmail to avoid double counting
            foreach ($participatingOrganisations as $organisationId => $organisationType) {
                $organisationStats->addActivity($organisationId, $organisationType, $killDate);
            }
        }

        return $organisationStats->filterByMinimumActiveDays($this->options->daysActive);
    }

    /**
     * @return array<int, string|int> [type, id]
     */
    private function getParticipatingOrganisations(object $killmail): array
    {
        $organisations = [];

        // Add victim's organisation
        [$type, $id] = $this->getOrganisationFromEntity($killmail->data->victim);
        if ($id !== 0) {
            $organisations[$id] = $type;
        }

        // Add unique attacker organisations (deduplicates automatically using array keys)
        foreach ($killmail->data->attackers as $attacker) {
            [$type, $id] = $this->getOrganisationFromEntity($attacker);
            if ($id !== 0) {
                $organisations[$id] = $type;
            }
        }

        return $organisations;
    }

    private function getTopOrganisations(OrganisationStatsCollection $organisationStats): OrganisationStatsCollection
    {
        return $organisationStats->topByKillCount($this->options->top);
    }

    private function updateMapSolarsystem(WormholeSystem $wormholeSystem, SystemAnalysisResult $analysisResult): void
    {
        $mapSolarsystem = MapSolarsystem::query()
            ->where('solarsystem_id', $wormholeSystem->solarsystem->id)
            ->where('map_id', $this->options->mapId)
            ->firstOrNew();

        $updatedNotes = $this->mergeNotesContent($mapSolarsystem->notes ?? '', $analysisResult->notesContent);

        $mapSolarsystem->fill([
            'solarsystem_id' => $wormholeSystem->solarsystem->id,
            'map_id' => $this->options->mapId,
            'notes' => $updatedNotes,
            'status' => $analysisResult->status,
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

    /**
     * @param  Collection<int, int>  $organisationIds
     */
    private function ensureNamesExist(Collection $organisationIds): void
    {
        $organisationIds->each($this->ensureNameExists(...));
    }

    /**
     * @throws InvalidArgumentException
     * @throws ConnectionException
     */
    private function ensureNameExists(int $id): void
    {
        if (Cache::memo()->get((string) $id)) {
            return;
        }

        if (Corporation::query()->whereId($id)->whereNotNull('name')->exists()) {
            Cache::memo()->put((string) $id, true);

            return;
        }

        if (Alliance::query()->whereId($id)->whereNotNull('name')->exists()) {
            Cache::memo()->put((string) $id, true);

            return;
        }

        $request = $this->esi->getNames([$id]);

        if ($request->failed()) {
            $this->error(sprintf('Failed to get name for ID %d', $id));

            return;
        }

        [$data] = $request->data;

        if ($data->category === NameCategory::Corporation) {
            Corporation::query()->updateOrCreate(
                ['id' => $data->id],
                ['name' => $data->name]
            );
            Cache::memo()->put((string) $data->id, true);

            return;
        }

        if ($data->category === NameCategory::Alliance) {
            Alliance::query()->updateOrCreate(
                ['id' => $data->id],
                ['name' => $data->name]
            );
            Cache::memo()->put((string) $data->id, true);

            return;
        }

        $this->error(sprintf('Unknown name category for ID %d: %s', $id, $data->category->value));
    }

    /**
     * @return array{'alliance'|'corporation'|'unknown', int} [type, id]
     */
    private function getOrganisationFromEntity(object $entity): array
    {
        if (isset($entity->alliance_id) && $entity->alliance_id > 0) {
            return ['alliance', $entity->alliance_id];
        }

        if (isset($entity->corporation_id) && $entity->corporation_id > 0) {
            return ['corporation', $entity->corporation_id];
        }

        return ['unknown', 0];
    }
}
