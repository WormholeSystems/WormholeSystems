<?php

declare(strict_types=1);

namespace App\Actions\MapTransfer;

use App\DTO\MapImportSummary;
use App\Enums\LifetimeStatus;
use App\Enums\ShipSize;
use App\Models\Alliance;
use App\Models\Character;
use App\Models\Corporation;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapConnection;
use App\Models\MapIgnoredSolarsystem;
use App\Models\MapRouteSolarsystem;
use App\Models\MapSolarsystem;
use App\Models\MapSolarsystemDetails;
use App\Models\Signature;
use App\Models\SignatureCategory;
use App\Models\SignatureType;
use App\Models\Solarsystem;
use App\Models\Wormhole;
use App\Support\Broadcasting\MapBroadcaster;
use Carbon\CarbonImmutable;
use Illuminate\Support\Facades\DB;
use InvalidArgumentException;
use Throwable;

final readonly class ImportMapAction
{
    public function __construct(private MapBroadcaster $mapBroadcaster) {}

    /**
     * Merge a parsed export payload into the map. Rows with a natural unique key
     * are updated in place; connections (which have none) are skipped when an
     * equivalent edge already exists. The owner access row is never touched.
     *
     * @param  array<string, mixed>  $payload
     *
     * @throws Throwable
     */
    public function handle(Map $map, array $payload): MapImportSummary
    {
        $summary = new MapImportSummary;

        DB::transaction(function () use ($map, $payload, $summary): void {
            $sections = $payload['sections'];
            $known_solarsystem_ids = $this->knownSolarsystemIds($sections);

            if (isset($sections['settings'])) {
                $this->importSettings($map, $sections['settings'], $known_solarsystem_ids, $summary);
            }

            if (isset($sections['access'])) {
                $this->importAccess($map, $sections['access'], $summary);
            }

            if (isset($sections['solarsystems'])) {
                $this->importSolarsystems($map, $sections['solarsystems'], $known_solarsystem_ids, $summary);
            }

            $connection_ids_by_index = [];

            if (isset($sections['connections'])) {
                $connection_ids_by_index = $this->importConnections($map, $sections['connections'], $summary);
            }

            if (isset($sections['signatures'])) {
                $this->importSignatures($map, $sections['signatures'], $connection_ids_by_index, $summary);
            }

            if (isset($sections['routes'])) {
                $this->importRoutes($map, $sections['routes'], $known_solarsystem_ids, $summary);
            }
        }, 5);

        $this->mapBroadcaster->resync($map->id);

        return $summary;
    }

    /**
     * Every solarsystem id referenced anywhere in the payload that exists in the
     * local static data, so unknown ids can be skipped instead of hitting FKs.
     *
     * @param  array<string, mixed>  $sections
     * @return array<int, bool>
     */
    private function knownSolarsystemIds(array $sections): array
    {
        $ids = collect([$sections['settings']['home_solarsystem_id'] ?? null])
            ->merge($this->rallySolarsystemIds($sections['settings'] ?? []))
            ->merge(collect($sections['solarsystems'] ?? [])->pluck('solarsystem_id'))
            ->merge(collect($sections['routes']['route_solarsystems'] ?? [])->pluck('solarsystem_id'))
            ->merge(collect($sections['routes']['ignored_solarsystems'] ?? [])->pluck('solarsystem_id'))
            ->filter()
            ->unique();

        return Solarsystem::query()
            ->whereIn('id', $ids)
            ->pluck('id')
            ->mapWithKeys(fn (int $id): array => [$id => true])
            ->all();
    }

    /**
     * @param  array<string, mixed>  $settings
     * @param  array<int, bool>  $known_solarsystem_ids
     */
    private function importSettings(Map $map, array $settings, array $known_solarsystem_ids, MapImportSummary $summary): void
    {
        $map->update([
            'name' => $settings['name'],
            'layout' => $settings['layout'],
            'allow_layout_override' => $settings['allow_layout_override'],
            'constant_width_enabled' => $settings['constant_width_enabled'],
            'bookmark_format_wormhole' => $settings['bookmark_format_wormhole'],
            'bookmark_format_kspace' => $settings['bookmark_format_kspace'],
            'bookmark_format_return' => $settings['bookmark_format_return'],
            'bookmark_alias_scheme' => $settings['bookmark_alias_scheme'],
            'bookmark_ignored_alias' => $settings['bookmark_ignored_alias'] ?? '',
            'home_solarsystem_id' => $this->knownOrNull($settings['home_solarsystem_id'], $known_solarsystem_ids),
            'rally_solarsystem_ids' => collect($this->rallySolarsystemIds($settings))
                ->filter(fn (int $solarsystem_id): bool => isset($known_solarsystem_ids[$solarsystem_id]))
                ->unique()
                ->take(-config()->integer('map.max_rally_points'))
                ->values()
                ->all(),
        ]);

        $summary->updated('settings');
    }

    /**
     * @param  list<array<string, mixed>>  $entries
     */
    private function importAccess(Map $map, array $entries, MapImportSummary $summary): void
    {
        $owner = $map->mapOwner;

        foreach ($entries as $entry) {
            $accessible_type = match ($entry['entity_type']) {
                'character' => Character::class,
                'corporation' => Corporation::class,
                'alliance' => Alliance::class,
                default => throw new InvalidArgumentException('Unknown access entity type.'),
            };

            $accessible_type::query()->firstOrCreate(
                ['id' => $entry['entity_id']],
                ['name' => $entry['entity_name']],
            );

            $existing = MapAccess::query()
                ->where('map_id', $map->id)
                ->where('accessible_type', $accessible_type)
                ->where('accessible_id', $entry['entity_id'])
                ->first();

            $is_owner_entity = $owner instanceof MapAccess
                && $owner->accessible_type === $accessible_type
                && $owner->accessible_id === (int) $entry['entity_id'];

            if ($existing?->is_owner || $is_owner_entity) {
                $summary->skipped('access');

                continue;
            }

            $attributes = [
                'permission' => $entry['permission'],
                'expires_at' => $entry['expires_at'] !== null ? CarbonImmutable::parse($entry['expires_at']) : null,
                'is_owner' => false,
            ];

            if ($existing instanceof MapAccess) {
                $existing->update($attributes);
                $summary->updated('access');
            } else {
                MapAccess::query()->create([
                    'map_id' => $map->id,
                    'accessible_type' => $accessible_type,
                    'accessible_id' => $entry['entity_id'],
                    ...$attributes,
                ]);
                $summary->created('access');
            }
        }
    }

    /**
     * Details first (they are the superset), then placements for entries that
     * carry canvas positions. Batched with upserts because alliance-scale
     * exports carry thousands of systems.
     *
     * @param  list<array<string, mixed>>  $entries
     * @param  array<int, bool>  $known_solarsystem_ids
     */
    private function importSolarsystems(Map $map, array $entries, array $known_solarsystem_ids, MapImportSummary $summary): void
    {
        $importable = [];

        foreach ($entries as $entry) {
            if (isset($known_solarsystem_ids[$entry['solarsystem_id']])) {
                $importable[$entry['solarsystem_id']] = $entry;
            } else {
                $summary->skipped('systems');
            }
        }

        if ($importable === []) {
            return;
        }

        $ids = array_keys($importable);
        $existing_details = $map->mapSolarsystemDetails()->whereIn('solarsystem_id', $ids)->pluck('solarsystem_id')->flip()->all();
        $existing_placements = $map->mapSolarsystems()->whereIn('solarsystem_id', $ids)->pluck('solarsystem_id')->flip()->all();

        foreach (array_chunk($importable, 500) as $chunk) {
            MapSolarsystemDetails::query()->upsert(
                array_map(fn (array $entry): array => [
                    'map_id' => $map->id,
                    'solarsystem_id' => $entry['solarsystem_id'],
                    'status' => $entry['status'],
                    'occupier_alias' => $entry['occupier_alias'],
                    'notes' => $entry['notes'],
                ], $chunk),
                ['map_id', 'solarsystem_id'],
                ['status', 'occupier_alias', 'notes'],
            );
        }

        $placed = array_values(array_filter(
            $importable,
            fn (array $entry): bool => $entry['position_x'] !== null && $entry['position_y'] !== null,
        ));

        if ($placed !== []) {
            $details_ids = $map->mapSolarsystemDetails()
                ->whereIn('solarsystem_id', array_column($placed, 'solarsystem_id'))
                ->pluck('id', 'solarsystem_id');

            foreach (array_chunk($placed, 500) as $chunk) {
                MapSolarsystem::query()->upsert(
                    array_map(fn (array $entry): array => [
                        'map_id' => $map->id,
                        'solarsystem_id' => $entry['solarsystem_id'],
                        'map_solarsystem_details_id' => $details_ids[$entry['solarsystem_id']],
                        'alias' => $entry['alias'],
                        'position_x' => (int) $entry['position_x'],
                        'position_y' => (int) $entry['position_y'],
                        'pinned' => (bool) ($entry['pinned'] ?? false),
                    ], $chunk),
                    ['map_id', 'solarsystem_id'],
                    ['map_solarsystem_details_id', 'alias', 'position_x', 'position_y', 'pinned'],
                );
            }
        }

        foreach ($importable as $solarsystem_id => $entry) {
            $has_position = $entry['position_x'] !== null && $entry['position_y'] !== null;
            $existed = $has_position ? isset($existing_placements[$solarsystem_id]) : isset($existing_details[$solarsystem_id]);
            $existed ? $summary->updated('systems') : $summary->created('systems');
        }
    }

    /**
     * @param  list<array<string, mixed>>  $entries
     * @return array<int, int> connection index in the file to local connection id
     */
    private function importConnections(Map $map, array $entries, MapImportSummary $summary): array
    {
        $placement_ids = $map->mapSolarsystems()->pluck('id', 'solarsystem_id');
        $existing = $map->mapConnections()->get(['id', 'from_map_solarsystem_id', 'to_map_solarsystem_id']);
        $connection_ids_by_index = [];

        foreach ($entries as $index => $entry) {
            $from_id = $placement_ids[$entry['from_solarsystem_id']] ?? null;
            $to_id = $placement_ids[$entry['to_solarsystem_id']] ?? null;

            if ($from_id === null || $to_id === null) {
                $summary->skipped('connections');

                continue;
            }

            $duplicate = $existing->first(
                fn (MapConnection $connection): bool => ($connection->from_map_solarsystem_id === $from_id && $connection->to_map_solarsystem_id === $to_id)
                    || ($connection->from_map_solarsystem_id === $to_id && $connection->to_map_solarsystem_id === $from_id)
            );

            if ($duplicate instanceof MapConnection) {
                $connection_ids_by_index[$index] = $duplicate->id;
                $summary->skipped('connections');

                continue;
            }

            $wormhole = $entry['wormhole'] !== null
                ? Wormhole::query()->where('name', $entry['wormhole'])->first()
                : null;
            $wormhole_ship_size = ShipSize::fromWormhole($wormhole);

            $connection = MapConnection::query()->create([
                'map_id' => $map->id,
                'from_map_solarsystem_id' => $from_id,
                'to_map_solarsystem_id' => $to_id,
                'wormhole_id' => $wormhole?->id,
                'type' => $entry['type'],
                'mass_status' => $entry['mass_status'],
                'ship_size' => $wormhole_ship_size instanceof ShipSize ? $wormhole_ship_size->value : $entry['ship_size'],
                'lifetime' => $entry['lifetime'],
                'lifetime_updated_at' => $entry['lifetime_updated_at'],
                'connected_at' => $entry['connected_at'] ?? now(),
                'preserve_mass' => $entry['preserve_mass'],
            ]);

            $existing->push($connection);
            $connection_ids_by_index[$index] = $connection->id;
            $summary->created('connections');
        }

        return $connection_ids_by_index;
    }

    /**
     * @param  list<array<string, mixed>>  $entries
     * @param  array<int, int>  $connection_ids_by_index
     */
    private function importSignatures(Map $map, array $entries, array $connection_ids_by_index, MapImportSummary $summary): void
    {
        $placement_ids = $map->mapSolarsystems()->pluck('id', 'solarsystem_id');

        foreach ($entries as $entry) {
            $placement_id = $placement_ids[$entry['solarsystem_id']] ?? null;

            if ($placement_id === null) {
                $summary->skipped('signatures');

                continue;
            }

            $connection_id = $entry['connection_index'] !== null
                ? ($connection_ids_by_index[$entry['connection_index']] ?? null)
                : null;

            $attributes = [
                'signature_category_id' => $entry['category'] !== null
                    ? SignatureCategory::query()->where('code', $entry['category'])->first()?->id
                    : null,
                'signature_type_id' => $entry['type_name'] !== null
                    ? SignatureType::query()->where('name', $entry['type_name'])->first()?->id
                    : null,
                'wormhole_id' => $entry['wormhole'] !== null
                    ? Wormhole::query()->where('name', $entry['wormhole'])->first()?->id
                    : null,
                'map_connection_id' => $connection_id,
                'raw_type_name' => $entry['raw_type_name'],
                'mass_status' => $entry['mass_status'],
                'ship_size' => $entry['ship_size'],
                'lifetime' => $entry['lifetime'] ?? LifetimeStatus::Healthy->value,
                'lifetime_updated_at' => $entry['lifetime_updated_at'],
            ];

            if ($entry['signature_id'] !== null) {
                $signature = Signature::query()->updateOrCreate(
                    ['map_solarsystem_id' => $placement_id, 'signature_id' => $entry['signature_id']],
                    $attributes,
                );

                $signature->wasRecentlyCreated ? $summary->created('signatures') : $summary->updated('signatures');

                continue;
            }

            $existing = Signature::query()
                ->where('map_solarsystem_id', $placement_id)
                ->whereNull('signature_id')
                ->when(
                    $connection_id !== null,
                    fn ($query) => $query->where('map_connection_id', $connection_id),
                    fn ($query) => $query->whereNull('map_connection_id'),
                )
                ->first();

            if ($existing instanceof Signature) {
                $existing->update($attributes);
                $summary->updated('signatures');
            } else {
                Signature::query()->create([
                    'map_solarsystem_id' => $placement_id,
                    'signature_id' => null,
                    ...$attributes,
                ]);
                $summary->created('signatures');
            }
        }
    }

    /**
     * @param  array<string, list<array<string, mixed>>>  $routes
     * @param  array<int, bool>  $known_solarsystem_ids
     */
    private function importRoutes(Map $map, array $routes, array $known_solarsystem_ids, MapImportSummary $summary): void
    {
        foreach ($routes['route_solarsystems'] as $entry) {
            if (! isset($known_solarsystem_ids[$entry['solarsystem_id']])) {
                $summary->skipped('routes');

                continue;
            }

            $route = MapRouteSolarsystem::query()->updateOrCreate(
                ['map_id' => $map->id, 'solarsystem_id' => $entry['solarsystem_id']],
                ['is_pinned' => $entry['is_pinned']],
            );

            $route->wasRecentlyCreated ? $summary->created('routes') : $summary->updated('routes');
        }

        foreach ($routes['ignored_solarsystems'] as $entry) {
            if (! isset($known_solarsystem_ids[$entry['solarsystem_id']])) {
                $summary->skipped('routes');

                continue;
            }

            $ignored = MapIgnoredSolarsystem::query()->firstOrCreate([
                'map_id' => $map->id,
                'solarsystem_id' => $entry['solarsystem_id'],
            ]);

            if ($ignored->wasRecentlyCreated) {
                $summary->created('routes');
            }
        }
    }

    /**
     * @param  array<int, bool>  $known_solarsystem_ids
     */
    /**
     * Older exports carry a single rally point under `rally_solarsystem_id`.
     *
     * @param  array<string, mixed>  $settings
     * @return list<int>
     */
    private function rallySolarsystemIds(array $settings): array
    {
        if (isset($settings['rally_solarsystem_ids'])) {
            return $settings['rally_solarsystem_ids'];
        }

        return isset($settings['rally_solarsystem_id']) ? [$settings['rally_solarsystem_id']] : [];
    }

    private function knownOrNull(?int $solarsystem_id, array $known_solarsystem_ids): ?int
    {
        return $solarsystem_id !== null && isset($known_solarsystem_ids[$solarsystem_id])
            ? $solarsystem_id
            : null;
    }
}
