<?php

declare(strict_types=1);

namespace App\Console\Commands;

use App\Utilities\CCPRounding;
use Closure;
use Illuminate\Console\Command;
use Illuminate\Support\Facades\Concurrency;
use Illuminate\Support\Facades\File;
use Illuminate\Support\Facades\Storage;
use JsonException;
use NicolasKion\Esi\Enums\NameCategory;
use NicolasKion\Esi\Esi;
use NicolasKion\Esi\Facades\Esi as EsiFacade;
use NicolasKion\SDE\Data\Dto\ConstellationDto;
use NicolasKion\SDE\Data\Dto\Position2dDto;
use NicolasKion\SDE\Data\Dto\RegionDto;
use NicolasKion\SDE\Data\Dto\SolarsystemDto;
use NicolasKion\SDE\Data\Dto\StargateDto;
use NicolasKion\SDE\Data\Dto\StationDto;
use NicolasKion\SDE\Data\Dto\StationOperationDto;
use NicolasKion\SDE\Data\Dto\StationServiceDto;
use NicolasKion\SDE\Support\JSONL;
use NicolasKion\SDE\Support\UniverseHelpers;
use RuntimeException;
use Symfony\Component\Console\Helper\ProgressIndicator;
use Throwable;

use function array_chunk;
use function array_filter;
use function array_flip;
use function array_map;
use function array_merge;
use function array_unique;
use function array_values;
use function collect;
use function explode;
use function file_get_contents;
use function hrtime;
use function in_array;
use function is_array;
use function is_string;
use function iterator_to_array;
use function ksort;
use function max;
use function mb_trim;
use function memory_get_peak_usage;
use function memory_get_usage;
use function round;
use function sort;
use function sprintf;
use function usort;

final class GenerateStaticDataCommand extends Command
{
    private const int SERVICE_SECURITY_OFFICE = 27;

    private const int CORPORATION_CONCORD = 1000125;

    private const int CORPORATION_DED = 1000137;

    private const int ESI_TICKER_CONCURRENCY_CHUNK_SIZE = 10;

    protected $signature = 'generate:static-data {--skip-sovereignty}';

    protected $description = 'Generate static data files for client-side routing';

    /**
     * @throws JsonException
     * @throws Throwable
     */
    public function handle(): int
    {
        $this->info('Generating static data files...');

        $resourcesPath = resource_path('static');
        File::ensureDirectoryExists($resourcesPath);

        $storagePath = storage_path('app/static');
        File::ensureDirectoryExists($storagePath);

        $data = $this->profileStep('Load source datasets', function (): array {
            $regionsById = $this->loadRegions();
            $constellationsById = $this->loadConstellations($regionsById);
            $solarsystemRows = $this->loadSolarsystemRows();

            $securityBySolarsystem = [];
            $solarsystemNamesById = [];
            foreach ($solarsystemRows as $solarsystemRow) {
                $securityBySolarsystem[$solarsystemRow->id] = $solarsystemRow->securityStatus;
                $solarsystemNamesById[$solarsystemRow->id] = $solarsystemRow->name;
            }

            [$solarsystemServices, $solarsystemHasStations] = $this->computeSolarsystemStationData(
                $this->loadOperationServices(),
                $securityBySolarsystem,
            );

            return [
                'regionsById' => $regionsById,
                'constellationsById' => $constellationsById,
                'solarsystemRows' => $solarsystemRows,
                'solarsystemNamesById' => $solarsystemNamesById,
                'solarsystemServices' => $solarsystemServices,
                'solarsystemHasStations' => $solarsystemHasStations,
                'wormholesByName' => $this->loadWormholesByName(),
                'wormholeEffectsByName' => $this->loadWormholeEffectsByName(),
                'wormholeSystemsById' => $this->loadWormholeSystemsById(),
                'joveObservatorySystems' => $this->loadJoveObservatorySystems(),
                'services' => $this->loadServices(),
            ];
        });

        $this->profileStep('Write regions', function () use ($data, $resourcesPath): bool {
            $this->writePlainJson(
                "$resourcesPath/regions.json",
                collect($data['regionsById'])->sortBy('id')->values()->all(),
            );

            return true;
        });

        $this->profileStep('Write constellations', function () use ($data, $resourcesPath): bool {
            $this->writePlainJson(
                "$resourcesPath/constellations.json",
                collect($data['constellationsById'])->sortBy('id')->values()->all(),
            );

            return true;
        });

        $this->profileStep('Write solarsystems', function () use ($data, $resourcesPath): bool {
            $this->writePlainJson(
                "$resourcesPath/solarsystems.json",
                $this->formatSolarsystems(
                    $data['solarsystemRows'],
                    $data['regionsById'],
                    $data['constellationsById'],
                    $data['solarsystemServices'],
                    $data['solarsystemHasStations'],
                    $data['wormholeSystemsById'],
                    $data['wormholesByName'],
                    $data['wormholeEffectsByName'],
                    $data['joveObservatorySystems'],
                ),
            );

            return true;
        });

        $this->profileStep('Write connections', function () use ($data, $resourcesPath): bool {
            $this->writePlainJson(
                "$resourcesPath/connections.json",
                $this->buildConnections($data['solarsystemNamesById']),
            );

            return true;
        });

        $this->profileStep('Write services', function () use ($data, $resourcesPath): bool {
            $this->writePlainJson(
                "$resourcesPath/services.json",
                $data['services'],
            );

            return true;
        });

        if (! $this->option('skip-sovereignty')) {
            $this->profileStep('Write sovereignty', function () use ($storagePath): bool {
                $this->writeCompressedJson(
                    "$storagePath/sovereignty.json.gz",
                    $this->fetchSovereigntyFromEsi(app(Esi::class)),
                );

                return true;
            });
            $this->info("✓ Sovereignty data written to $storagePath");
        } else {
            $this->warn('Skipping sovereignty generation (--skip-sovereignty).');
        }

        $this->info("✓ Static data written to $resourcesPath");

        return self::SUCCESS;
    }

    /**
     * @return array<int, array{id: int, name: string, type: string}>
     */
    private function loadRegions(): array
    {
        $regions = [];

        foreach (JSONL::lazy($this->sdePath('mapRegions.jsonl'), RegionDto::class) as $region) {
            $id = $region->id;

            $regions[$id] = [
                'id' => $id,
                'name' => $region->name,
                'type' => UniverseHelpers::determineAreaType($id),
            ];
        }

        return $regions;
    }

    /**
     * @param  array<int, array{id: int, name: string, type: string}>  $regionsById
     * @return array<int, array{id: int, name: string, type: string, region_id: int, region: array{id: int, name: string}}>
     */
    private function loadConstellations(array $regionsById): array
    {
        $constellations = [];

        foreach (JSONL::lazy($this->sdePath('mapConstellations.jsonl'), ConstellationDto::class) as $constellation) {
            $id = $constellation->id;
            $regionId = $constellation->regionId;
            $region = $regionsById[$regionId] ?? null;

            if ($region === null) {
                continue;
            }

            $constellations[$id] = [
                'id' => $id,
                'name' => $constellation->name,
                'type' => UniverseHelpers::determineAreaType($id),
                'region_id' => $regionId,
                'region' => [
                    'id' => $region['id'],
                    'name' => $region['name'],
                ],
            ];
        }

        return $constellations;
    }

    /**
     * @return array<int, SolarsystemDto>
     */
    private function loadSolarsystemRows(): array
    {
        return iterator_to_array(JSONL::lazy($this->sdePath('mapSolarSystems.jsonl'), SolarsystemDto::class));
    }

    /**
     * @return array<int, array{id: int, name: string}>
     */
    private function loadServices(): array
    {
        $services = [];

        foreach (JSONL::lazy($this->sdePath('stationServices.jsonl'), StationServiceDto::class) as $service) {
            $services[] = [
                'id' => $service->id,
                'name' => $service->serviceName,
            ];
        }

        usort($services, static fn (array $a, array $b): int => $a['id'] <=> $b['id']);

        return $services;
    }

    /**
     * @return array<int, int[]>
     */
    private function loadOperationServices(): array
    {
        $operationServices = [];

        foreach (JSONL::lazy($this->sdePath('stationOperations.jsonl'), StationOperationDto::class) as $operation) {
            $serviceIds = array_values(array_unique(array_map(static fn (int $serviceId): int => $serviceId, $operation->services)));
            sort($serviceIds);

            $operationServices[$operation->id] = $serviceIds;
        }

        return $operationServices;
    }

    /**
     * @param  array<int, int[]>  $operationServices
     * @param  array<int, float>  $securityBySolarsystem
     * @return array{array<int, int[]>, array<int, bool>}
     */
    private function computeSolarsystemStationData(array $operationServices, array $securityBySolarsystem): array
    {
        $solarsystemServices = [];
        $solarsystemHasStations = [];

        foreach (JSONL::lazy($this->sdePath('npcStations.jsonl'), StationDto::class) as $station) {
            $solarsystemHasStations[$station->solarSystemId] = true;

            if ($station->operationId === null) {
                continue;
            }

            if (($operationServices[$station->operationId] ?? []) === []) {
                continue;
            }

            $allowedServices = collect($operationServices[$station->operationId])
                ->filter(fn (int $serviceId): bool => $this->isServiceAvailable($serviceId, $station->ownerId, $securityBySolarsystem[$station->solarSystemId] ?? 1.0))
                ->values()
                ->all();

            if ($allowedServices === []) {
                continue;
            }

            if (! isset($solarsystemServices[$station->solarSystemId])) {
                $solarsystemServices[$station->solarSystemId] = [];
            }

            $solarsystemServices[$station->solarSystemId] = array_values(
                array_unique(array_merge($solarsystemServices[$station->solarSystemId], $allowedServices))
            );
        }

        foreach (array_keys($solarsystemServices) as $id) {
            sort($solarsystemServices[$id]);
        }

        return [$solarsystemServices, $solarsystemHasStations];
    }

    private function isServiceAvailable(int $serviceId, ?int $ownerId, float $security): bool
    {
        if ($serviceId === self::SERVICE_SECURITY_OFFICE) {
            $rounded = CCPRounding::roundSecurity($security);

            return $rounded < 0.5 && in_array($ownerId, [self::CORPORATION_CONCORD, self::CORPORATION_DED], true);
        }

        return true;
    }

    /**
     * @return array<string, array<string, mixed>>
     *
     * @throws JsonException
     */
    private function loadWormholesByName(): array
    {
        $path = database_path('seeders/data/wormholes.json');
        $this->assertFileExists($path);

        $decoded = json_decode($this->readFileContents($path), true, flags: JSON_THROW_ON_ERROR);

        if (! is_array($decoded)) {
            throw new RuntimeException('Invalid wormholes.json payload.');
        }

        return $decoded;
    }

    /**
     * @return array<string, array<string, mixed>>
     *
     * @throws JsonException
     */
    private function loadWormholeEffectsByName(): array
    {
        $path = database_path('seeders/data/wormhole_effects.json');
        $this->assertFileExists($path);

        $decoded = json_decode($this->readFileContents($path), true, flags: JSON_THROW_ON_ERROR);

        if (! is_array($decoded)) {
            throw new RuntimeException('Invalid wormhole_effects.json payload.');
        }

        return $decoded;
    }

    /**
     * @return array<int, array{class: int, effect: string|null, statics: string[]}>
     */
    private function loadWormholeSystemsById(): array
    {
        $path = database_path('seeders/data/wormhole_systems.csv');
        $this->assertFileExists($path);

        $handle = fopen($path, 'rb');
        if ($handle === false) {
            throw new RuntimeException('Unable to open wormhole_systems.csv.');
        }

        $header = fgetcsv($handle, escape: '\\');
        if (! is_array($header)) {
            fclose($handle);
            throw new RuntimeException('Invalid wormhole_systems.csv header.');
        }

        $indexByColumn = array_flip($header);
        $systems = [];

        while (($row = fgetcsv($handle, escape: '\\')) !== false) {

            $id = (int) $row[$indexByColumn['id']];
            $effect = mb_trim((string) $row[$indexByColumn['effect']]);
            $staticsValue = mb_trim((string) $row[$indexByColumn['statics']]);

            $systems[$id] = [
                'class' => (int) $row[$indexByColumn['class']],
                'effect' => $effect !== '' ? $effect : null,
                'statics' => $staticsValue === ''
                    ? []
                    : array_values(array_filter(array_map(mb_trim(...), explode(',', $staticsValue)))),
            ];
        }

        fclose($handle);

        return $systems;
    }

    /**
     * @return array<string, true>
     */
    private function loadJoveObservatorySystems(): array
    {
        $path = base_path('vendor/nicolaskion/sde/src/Data/jove_observatories.php');
        $this->assertFileExists($path);

        $regions = require $path;
        $systems = [];

        foreach ($regions as $regionSystems) {
            if (! is_array($regionSystems)) {
                continue;
            }

            foreach ($regionSystems as $systemName) {
                if (! is_string($systemName)) {
                    continue;
                }
                $systems[$systemName] = true;
            }
        }

        return $systems;
    }

    /**
     * @param  array<int, SolarsystemDto>  $solarsystemRows
     * @param  array<int, array{id: int, name: string, type: string}>  $regionsById
     * @param  array<int, array{id: int, name: string, type: string, region_id: int, region: array{id: int, name: string}}>  $constellationsById
     * @param  array<int, int[]>  $solarsystemServices
     * @param  array<int, bool>  $solarsystemHasStations
     * @param  array<int, array{class: int, effect: string|null, statics: string[]}>  $wormholeSystemsById
     * @param  array<string, array<string, mixed>>  $wormholesByName
     * @param  array<string, array<string, mixed>>  $wormholeEffectsByName
     * @param  array<string, true>  $joveObservatorySystems
     * @return array<int, array<string, mixed>>
     */
    private function formatSolarsystems(
        array $solarsystemRows,
        array $regionsById,
        array $constellationsById,
        array $solarsystemServices,
        array $solarsystemHasStations,
        array $wormholeSystemsById,
        array $wormholesByName,
        array $wormholeEffectsByName,
        array $joveObservatorySystems
    ): array {
        $solarsystems = [];

        foreach ($solarsystemRows as $solarsystemRow) {
            $wormholeSystem = $wormholeSystemsById[$solarsystemRow->id] ?? null;
            $region = $regionsById[$solarsystemRow->regionId] ?? null;
            $constellation = $constellationsById[$solarsystemRow->constellationId] ?? null;
            if ($region === null) {
                continue;
            }
            if ($constellation === null) {
                continue;
            }

            $solarsystems[] = [
                'id' => $solarsystemRow->id,
                'name' => $solarsystemRow->name,
                'region_id' => $solarsystemRow->regionId,
                'constellation_id' => $solarsystemRow->constellationId,
                'class' => $wormholeSystem['class'] ?? null,
                'security' => CCPRounding::roundSecurity($solarsystemRow->securityStatus),
                'type' => UniverseHelpers::determineAreaType($solarsystemRow->id),
                'region' => [
                    'id' => $region['id'],
                    'name' => $region['name'],
                ],
                'constellation' => [
                    'id' => $constellation['id'],
                    'name' => $constellation['name'],
                ],
                'has_jove_observatory' => isset($joveObservatorySystems[$solarsystemRow->name]),
                'has_stations' => isset($solarsystemHasStations[$solarsystemRow->id]),
                'services' => $solarsystemServices[$solarsystemRow->id] ?? [],
                'statics' => $this->formatStatics($wormholeSystem, $wormholesByName),
                'effect' => $this->getWormholeEffectsFromData($wormholeSystem, $wormholeEffectsByName),
                'position2D' => $this->getPosition2dArrayFromRow($solarsystemRow),
            ];
        }

        usort($solarsystems, static fn (array $a, array $b): int => $a['id'] <=> $b['id']);

        return $solarsystems;
    }

    /**
     * @param  array{class: int, effect: string|null, statics: string[]}|null  $wormholeSystem
     * @param  array<string, array<string, mixed>>  $wormholesByName
     * @return array<int, array<string, mixed>>|null
     */
    private function formatStatics(?array $wormholeSystem, array $wormholesByName): ?array
    {
        if ($wormholeSystem === null || $wormholeSystem['statics'] === []) {
            return null;
        }

        $statics = [];

        foreach ($wormholeSystem['statics'] as $staticName) {
            $wormhole = $wormholesByName[$staticName] ?? null;

            if ($wormhole === null) {
                continue;
            }

            $statics[] = [
                'id' => (int) ($wormhole['typeID'] ?? 0),
                'leads_to' => (string) ($wormhole['dest'] ?? ''),
                'name' => $staticName,
                'maximum_lifetime' => (int) (($wormhole['lifetime'] ?? 0) * 60 * 60),
                'maximum_jump_mass' => (int) ($wormhole['max_mass_per_jump'] ?? 0),
                'total_mass' => (int) ($wormhole['total_mass'] ?? 0),
            ];
        }

        if ($statics === []) {
            return null;
        }

        return $statics;
    }

    /**
     * @param  array{class: int, effect: string|null, statics: string[]}|null  $wormholeSystem
     * @param  array<string, array<string, mixed>>  $wormholeEffectsByName
     */
    private function getWormholeEffectsFromData(?array $wormholeSystem, array $wormholeEffectsByName): ?array
    {
        if ($wormholeSystem === null || $wormholeSystem['effect'] === null) {
            return null;
        }

        $effects = $wormholeEffectsByName[$wormholeSystem['effect']] ?? null;

        if ($effects === null || ! isset($effects['Buffs'], $effects['Debuffs'])) {
            return null;
        }

        ['Buffs' => $buffs, 'Debuffs' => $debuffs] = $effects;

        $strength = $wormholeSystem['class'] - 1;

        $buffValues = collect($buffs)
            ->map(static fn (array $strengths, string $name): array => [
                'name' => $name,
                'strength' => $strengths[$strength] ?? null,
                'type' => 'Buff',
            ])
            ->merge(
                collect($debuffs)
                    ->map(static fn (array $strengths, string $name): array => [
                        'name' => $name,
                        'strength' => $strengths[$strength] ?? null,
                        'type' => 'Debuff',
                    ])
            )
            ->values()
            ->all();

        return [
            'id' => crc32($wormholeSystem['effect']),
            'name' => $wormholeSystem['effect'],
            'effects' => $buffValues,
        ];
    }

    private function getPosition2dArrayFromRow(SolarsystemDto $solarsystemRow): ?array
    {
        $position2d = $solarsystemRow->position2d;

        if (! $position2d instanceof Position2dDto) {
            return null;
        }

        return [
            'x' => $position2d->x,
            'y' => $position2d->y,
        ];
    }

    /**
     * @param  array<int, string>  $solarsystemNamesById
     * @return array<int, int[]>
     */
    private function buildConnections(array $solarsystemNamesById): array
    {
        $connections = [];

        foreach (JSONL::lazy($this->sdePath('mapStargates.jsonl'), StargateDto::class) as $stargate) {
            $fromSolarsystemId = $stargate->solarSystemId;
            $toSolarsystemId = $stargate->destination->solarSystemId;

            if (($solarsystemNamesById[$fromSolarsystemId] ?? null) === 'Zarzakh') {
                continue;
            }

            if (! isset($connections[$fromSolarsystemId])) {
                $connections[$fromSolarsystemId] = [];
            }

            $connections[$fromSolarsystemId][] = $toSolarsystemId;
        }

        ksort($connections);

        foreach ($connections as $fromSolarsystemId => $toSolarsystemIds) {
            $unique = array_values(array_unique($toSolarsystemIds));
            sort($unique);
            $connections[$fromSolarsystemId] = $unique;
        }

        return $connections;
    }

    /**
     * @return array<int, array{id: int, alliance: ?array{id: int, name: string, ticker: string}, corporation: ?array{id: int, name: string, ticker: string}, faction: ?array{id: int, name: string}}>
     */
    private function fetchSovereigntyFromEsi(Esi $esi): array
    {
        $result = $esi->getSovereignty();

        if ($result->failed()) {
            $this->warn('Failed to fetch sovereignty from ESI, writing empty data set.');

            return [];
        }

        if (! is_array($result->data)) {
            return [];
        }

        $sovereignties = collect($result->data);

        $allianceIds = $sovereignties->pluck('alliance_id')->filter()->unique()->values()->all();
        $corporationIds = $sovereignties->pluck('corporation_id')->filter()->unique()->values()->all();
        $factionIds = $sovereignties->pluck('faction_id')->filter()->unique()->values()->all();

        $namesById = $this->fetchNamesById($esi, array_values(array_unique([...$allianceIds, ...$corporationIds, ...$factionIds])));
        $allianceTickersById = $this->fetchAllianceTickersById($esi, $allianceIds);
        $corporationTickersById = $this->fetchCorporationTickersById($esi, $corporationIds);

        $payload = [];

        foreach ($sovereignties as $sovereignty) {
            $payload[$sovereignty->system_id] = [
                'id' => $sovereignty->system_id,
                'alliance' => $this->formatSovereigntyAlliance($sovereignty->alliance_id, $namesById, $allianceTickersById),
                'corporation' => $this->formatSovereigntyCorporation($sovereignty->corporation_id, $namesById, $corporationTickersById),
                'faction' => $this->formatSovereigntyFaction($sovereignty->faction_id, $namesById),
            ];
        }

        ksort($payload);

        return $payload;
    }

    /**
     * @param  int[]  $ids
     * @return array<int, string>
     */
    private function fetchNamesById(Esi $esi, array $ids): array
    {
        if ($ids === []) {
            return [];
        }

        $namesById = [];

        foreach (array_chunk($ids, 1000) as $chunk) {
            $result = $esi->getNames($chunk);

            if ($result->failed()) {
                continue;
            }

            foreach ($result->data as $name) {
                if (! in_array($name->category, [NameCategory::Alliance, NameCategory::Corporation, NameCategory::Faction], true)) {
                    continue;
                }

                $namesById[$name->id] = $name->name;
            }
        }

        return $namesById;
    }

    /**
     * @param  int[]  $allianceIds
     * @return array<int, string>
     */
    private function fetchAllianceTickersById(Esi $esi, array $allianceIds): array
    {
        $tickersById = $this->fetchAllianceTickersConcurrently($allianceIds);

        foreach ($allianceIds as $allianceId) {
            if (($tickersById[$allianceId] ?? '') !== '') {
                continue;
            }

            $result = $esi->getAlliance($allianceId);
            if ($result->failed()) {
                continue;
            }
            if ($result->data === null) {
                continue;
            }

            $ticker = $result->data->ticker ?? '';
            if ($ticker !== '') {
                $tickersById[$allianceId] = $ticker;
            }
        }

        return $tickersById;
    }

    /**
     * @param  int[]  $corporationIds
     * @return array<int, string>
     */
    private function fetchCorporationTickersById(Esi $esi, array $corporationIds): array
    {
        $tickersById = $this->fetchCorporationTickersConcurrently($corporationIds);

        foreach ($corporationIds as $corporationId) {
            if (($tickersById[$corporationId] ?? '') !== '') {
                continue;
            }

            $result = $esi->getCorporation($corporationId);
            if ($result->failed()) {
                continue;
            }
            if ($result->data === null) {
                continue;
            }

            $ticker = $result->data->ticker ?? '';
            if ($ticker !== '') {
                $tickersById[$corporationId] = $ticker;
            }
        }

        return $tickersById;
    }

    /**
     * @param  int[]  $allianceIds
     * @return array<int, string>
     */
    private function fetchAllianceTickersConcurrently(array $allianceIds): array
    {
        if ($allianceIds === []) {
            return [];
        }

        if (app()->runningUnitTests()) {
            return [];
        }

        $tickersById = [];

        foreach (array_chunk($allianceIds, self::ESI_TICKER_CONCURRENCY_CHUNK_SIZE) as $chunk) {
            $tasks = [];

            foreach ($chunk as $id) {
                $tasks[] = static function () use ($id): ?array {
                    $result = EsiFacade::getAlliance($id);

                    if ($result->failed()) {
                        return null;
                    }

                    return [
                        'id' => $id,
                        'ticker' => $result->data->ticker ?? null,
                    ];
                };
            }

            $results = Concurrency::run($tasks);

            foreach ($results as $tickerResult) {
                $tickersById[(int) $tickerResult['id']] = $tickerResult['ticker'];
            }
        }

        return $tickersById;
    }

    /**
     * @param  int[]  $corporationIds
     * @return array<int, string>
     */
    private function fetchCorporationTickersConcurrently(array $corporationIds): array
    {
        if ($corporationIds === []) {
            return [];
        }

        if (app()->runningUnitTests()) {
            return [];
        }

        $tickersById = [];

        foreach (array_chunk($corporationIds, self::ESI_TICKER_CONCURRENCY_CHUNK_SIZE) as $chunk) {
            $tasks = [];

            foreach ($chunk as $id) {
                $tasks[] = static function () use ($id): ?array {
                    $result = EsiFacade::getCorporation($id);

                    if ($result->failed()) {
                        return null;
                    }

                    return [
                        'id' => $id,
                        'ticker' => $result->data->ticker ?? null,
                    ];
                };
            }

            $results = Concurrency::run($tasks);

            foreach ($results as $tickerResult) {
                $tickersById[(int) $tickerResult['id']] = $tickerResult['ticker'];
            }
        }

        return $tickersById;
    }

    /**
     * @param  array<int, string>  $namesById
     * @param  array<int, string>  $tickersById
     * @return array{id: int, name: string, ticker: string}|null
     */
    private function formatSovereigntyAlliance(?int $allianceId, array $namesById, array $tickersById): ?array
    {
        if ($allianceId === null) {
            return null;
        }

        return [
            'id' => $allianceId,
            'name' => $namesById[$allianceId] ?? '',
            'ticker' => $tickersById[$allianceId] ?? '',
        ];
    }

    /**
     * @param  array<int, string>  $namesById
     * @param  array<int, string>  $tickersById
     * @return array{id: int, name: string, ticker: string}|null
     */
    private function formatSovereigntyCorporation(?int $corporationId, array $namesById, array $tickersById): ?array
    {
        if ($corporationId === null) {
            return null;
        }

        return [
            'id' => $corporationId,
            'name' => $namesById[$corporationId] ?? '',
            'ticker' => $tickersById[$corporationId] ?? '',
        ];
    }

    /**
     * @param  array<int, string>  $namesById
     * @return array{id: int, name: string}|null
     */
    private function formatSovereigntyFaction(?int $factionId, array $namesById): ?array
    {
        if ($factionId === null) {
            return null;
        }

        return [
            'id' => $factionId,
            'name' => $namesById[$factionId] ?? '',
        ];
    }

    private function sdePath(string $file): string
    {
        $path = Storage::path("sde/$file");
        $this->assertFileExists($path);

        return $path;
    }

    private function assertFileExists(string $path): void
    {
        if (! File::exists($path)) {
            throw new RuntimeException("Required file not found: $path");
        }
    }

    private function readFileContents(string $path): string
    {
        $this->assertFileExists($path);

        $contents = file_get_contents($path);
        if (! is_string($contents)) {
            throw new RuntimeException("Unable to read file: $path");
        }

        return $contents;
    }

    /**
     * @throws JsonException
     */
    private function writeCompressedJson(string $path, array $data): void
    {
        $encoded = json_encode($data, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
        $compressed = gzencode($encoded, 9);

        if ($compressed === false) {
            throw new RuntimeException("Failed to compress JSON for $path.");
        }

        File::put($path, $compressed);
    }

    /**
     * @throws JsonException
     */
    private function writePlainJson(string $path, array $data): void
    {
        $encoded = json_encode($data, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES | JSON_PRETTY_PRINT);
        File::put($path, $encoded);
    }

    /**
     * @template T
     *
     * @param  Closure(): T  $callback
     * @return T
     *
     * @throws Throwable
     */
    private function profileStep(string $label, Closure $callback): mixed
    {
        $spinner = new ProgressIndicator($this->output);
        $startTime = hrtime(true);
        $startMemory = memory_get_usage(true);
        $startPeak = memory_get_peak_usage(true);

        $spinner->start($label);

        try {
            $result = $callback();
        } catch (Throwable $throwable) {
            $spinner->finish("$label failed");
            throw $throwable;
        }

        $spinner->finish(sprintf(
            '%s done (%s, mem %s, peak +%s)',
            $label,
            $this->formatDuration(hrtime(true) - $startTime),
            $this->formatBytes(memory_get_usage(true) - $startMemory),
            $this->formatBytes(max(0, memory_get_peak_usage(true) - $startPeak)),
        ));

        return $result;
    }

    private function formatDuration(int $nanoseconds): string
    {
        if ($nanoseconds < 1_000_000_000) {
            return sprintf('%.1fms', $nanoseconds / 1_000_000);
        }

        return sprintf('%.2fs', $nanoseconds / 1_000_000_000);
    }

    private function formatBytes(int $bytes): string
    {
        $negative = $bytes < 0;
        $value = abs($bytes);
        $units = ['B', 'KB', 'MB', 'GB'];
        $unit = 0;

        while ($value >= 1024 && $unit < count($units) - 1) {
            $value /= 1024;
            $unit++;
        }

        return ($negative ? '-' : '+').sprintf('%.1f%s', round($value, 1), $units[$unit]);
    }
}
