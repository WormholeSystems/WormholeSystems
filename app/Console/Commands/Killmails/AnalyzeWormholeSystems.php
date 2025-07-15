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
        {--active-threshold=75 : Minimum number of kills to consider a system active}
        {--hostile-threshold=150 : Minimum number of kills to consider a system hostile}';

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

        progress('Analyzing wormhole systems', WormholeSystem::all(), fn (WormholeSystem $wormholeSystem) => $this->analyzeWormholeSystem($wormholeSystem));

        MapSolarsystemsUpdatedEvent::dispatch($this->argument('map'));

        return self::SUCCESS;
    }

    private function analyzeWormholeSystem(WormholeSystem $wormholeSystem): void
    {
        $map_id = (int) $this->argument('map');
        $days_ago = (int) $this->option('days-ago');
        $days_active = (int) $this->option('days-active');
        $top = (int) $this->option('top');
        +$active_threshold = (int) $this->option('active-threshold');
        $hostile_threshold = (int) $this->option('hostile-threshold');

        $solarsystem = $wormholeSystem->solarsystem;

        $killmails = $solarsystem->killmails()->where('time', '>=', now()->subDays($days_ago))->get();

        $top_groups = [];

        foreach ($killmails as $killmail) {
            $day = $killmail->time->format('Y-m-d');

            [$type, $id] = $this->getTypeAndIdFromGroup($killmail->data->victim);

            if ($id !== 0) {
                $this->addGroupToGroups($top_groups, $id, $type, $day);
            }

            foreach ($killmail->data->attackers as $attacker) {
                [$type, $id] = $this->getTypeAndIdFromGroup($attacker);

                if ($id !== 0) {
                    $this->addGroupToGroups($top_groups, $id, $type, $day);
                }
            }
        }

        $top_groups = collect($top_groups)
            ->filter(fn (array $group) => count($group['days']) >= $days_active)
            ->sortDesc()->take($top);

        $top_10_ids = $top_groups->keys();

        $this->ensureNamesExist($top_10_ids);

        $markdown = $top_groups->map($this->getEntityDetails(...))->implode("\n");

        $top_count = $top_groups->sum('kills');

        $status = match (true) {
            $top_count >= $hostile_threshold => MapSolarsystemStatus::Hostile,
            $top_count >= $active_threshold => MapSolarsystemStatus::Active,
            default => MapSolarsystemStatus::Unknown
        };

        $start = '<!-- killmails:start -->';
        $end = '<!-- killmails:end -->';

        $map_solarsystem = MapSolarsystem::query()
            ->where('solarsystem_id', $solarsystem->id)
            ->where('map_id', $map_id)
            ->firstOrNew();

        if (empty($markdown)) {
            $markdown = 'We could not find any groups that meet the criteria.';
        }

        $new_note = sprintf(
            "%s\nTop %d groups that were active for at least %d dys within the last %d days:\n\n%s\n\n%s\n",
            $start,
            $top,
            $days_active,
            $days_ago,
            $markdown,
            $end
        );

        $has_existing_note_block = preg_match('/<!-- killmails:start -->.*<!-- killmails:end -->/s', $map_solarsystem->notes ?? '');

        if ($has_existing_note_block) {
            $new_note = preg_replace(
                '/<!-- killmails:start -->.*<!-- killmails:end -->/s',
                $new_note,
                $map_solarsystem->notes
            );
        } else {
            $new_note = ($map_solarsystem->notes ?? '').$new_note;
        }

        $map_solarsystem->fill([
            'solarsystem_id' => $solarsystem->id,
            'map_id' => $map_id,
            'notes' => $new_note,
            'status' => $status,
        ]);

        $map_solarsystem->save();
    }

    private function ensureNamesExist(Collection $top_ids): void
    {
        $top_ids->each(/**
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

        $request = $this->esi->getNames([
            $id,
        ]);

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

    private function addGroupToGroups(
        array &$top_groups,
        int $id,
        string $type,
        string $day
    ): void {
        if (! isset($top_groups[$id])) {
            $top_groups[$id] = [
                'type' => $type,
                'kills' => 0,
                'days' => [],
            ];
        }
        $top_groups[$id]['kills']++;
        $top_groups[$id]['days'][$day] = true;
    }

    private function getTypeAndIdFromGroup(object $group): array
    {
        if (isset($group->alliance_id) && $group->alliance_id > 0) {
            return ['alliance', $group->alliance_id];
        }

        if (isset($group->corporation_id) && $group->corporation_id > 0) {
            return ['corporation', $group->corporation_id];
        }

        return ['unknown', 0];
    }

    private function getEntityDetails(array $group, int $id): string
    {
        $alliance = Alliance::query()->find($id);

        if ($alliance) {
            return $this->getAllianceDetails($alliance, $group['kills']);
        }

        $corporation = Corporation::query()->find($id);
        if ($corporation) {
            return $this->getCorporationDetails($corporation, $group['kills']);
        }

        return sprintf('Unknown entity with ID %d and %d kills', $id, $group['kills']);
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
