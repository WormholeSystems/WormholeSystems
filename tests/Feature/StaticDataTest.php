<?php

declare(strict_types=1);

use App\Models\User;
use Illuminate\Support\Facades\Artisan;
use Illuminate\Support\Facades\File;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Storage;
use NicolasKion\Esi\DTO\EsiResult;
use NicolasKion\Esi\DTO\Name;
use NicolasKion\Esi\DTO\Sovereignty as EsiSovereignty;
use NicolasKion\Esi\Enums\NameCategory;
use NicolasKion\Esi\Esi;

use function Pest\Laravel\actingAs;

beforeEach(function () {
    $resourcesPath = resource_path('static');
    $storagePath = storage_path('app/static');
    $sdePath = Storage::path('sde');

    File::ensureDirectoryExists($resourcesPath);
    File::ensureDirectoryExists($storagePath);
    File::ensureDirectoryExists($sdePath);

    writeJsonl("$sdePath/mapRegions.jsonl", [
        [
            '_key' => 10000001,
            'name' => ['en' => 'Test Region'],
            'constellationIDs' => [20000001],
        ],
    ]);

    writeJsonl("$sdePath/mapConstellations.jsonl", [
        [
            '_key' => 20000001,
            'regionID' => 10000001,
            'name' => ['en' => 'Test Constellation'],
            'solarSystemIDs' => [30000001],
        ],
    ]);

    writeJsonl("$sdePath/mapSolarSystems.jsonl", [
        [
            '_key' => 30000001,
            'name' => ['en' => 'Test System'],
            'constellationID' => 20000001,
            'regionID' => 10000001,
            'securityStatus' => 0.4,
            'position' => ['x' => 0.0, 'y' => 0.0, 'z' => 0.0],
            'position2D' => ['x' => 1.0, 'y' => 2.0],
            'radius' => null,
            'securityClass' => null,
        ],
    ]);

    writeJsonl("$sdePath/stationServices.jsonl", [
        ['_key' => 5, 'serviceName' => ['en' => 'Reprocessing Plant']],
        ['_key' => 27, 'serviceName' => ['en' => 'Security Office']],
    ]);

    writeJsonl("$sdePath/stationOperations.jsonl", [
        ['_key' => 9, 'operationName' => ['en' => 'Test Operation'], 'services' => [5, 27]],
    ]);

    writeJsonl("$sdePath/npcStations.jsonl", [
        [
            '_key' => 60000001,
            'solarSystemID' => 30000001,
            'orbitID' => 40000001,
            'orbitIndex' => 1,
            'celestialIndex' => 1,
            'ownerID' => 1000125,
            'operationID' => 9,
            'useOperationName' => false,
            'typeID' => 1531,
        ],
    ]);

    writeJsonl("$sdePath/mapStargates.jsonl", [
        [
            '_key' => 50000001,
            'solarSystemID' => 30000001,
            'typeID' => 16,
            'position' => ['x' => 0.0, 'y' => 0.0, 'z' => 0.0],
            'destination' => ['stargateID' => 50000002, 'solarSystemID' => 30000002],
        ],
    ]);
});

afterEach(function () {
    $resourcesPath = resource_path('static');
    $storagePath = storage_path('app/static');
    $sdePath = Storage::path('sde');

    File::delete([
        "$resourcesPath/regions.json",
        "$resourcesPath/constellations.json",
        "$resourcesPath/solarsystems.json",
        "$resourcesPath/connections.json",
        "$resourcesPath/services.json",
        "$storagePath/sovereignty.json.gz",
        "$sdePath/mapRegions.jsonl",
        "$sdePath/mapConstellations.jsonl",
        "$sdePath/mapSolarSystems.jsonl",
        "$sdePath/stationServices.jsonl",
        "$sdePath/stationOperations.jsonl",
        "$sdePath/npcStations.jsonl",
        "$sdePath/mapStargates.jsonl",
    ]);
});

it('generates static data files from SDE and ESI', function () {
    $resourcesPath = resource_path('static');
    $storagePath = storage_path('app/static');

    Http::fake(function ($request) {
        if (str_contains($request->url(), '/alliances/99000001/')) {
            return Http::response(['ticker' => 'TALL'], 200);
        }

        if (str_contains($request->url(), '/corporations/98000001/')) {
            return Http::response(['ticker' => 'TCORP'], 200);
        }

        return Http::response([], 404);
    });

    $esi = $this->mock(Esi::class);
    $esi->shouldReceive('getSovereignty')
        ->once()
        ->andReturn(new EsiResult(data: [
            new EsiSovereignty(system_id: 30000001, alliance_id: 99000001, faction_id: 500001, corporation_id: 98000001),
        ]));

    $esi->shouldReceive('getNames')
        ->once()
        ->andReturn(new EsiResult(data: [
            new Name(category: NameCategory::Alliance, id: 99000001, name: 'Test Alliance'),
            new Name(category: NameCategory::Corporation, id: 98000001, name: 'Test Corporation'),
            new Name(category: NameCategory::Faction, id: 500001, name: 'Test Faction'),
        ]));

    $esi->shouldReceive('getAlliance')
        ->once()
        ->with(99000001)
        ->andReturn(new EsiResult(data: (object) ['ticker' => 'TALL']));

    $esi->shouldReceive('getCorporation')
        ->once()
        ->with(98000001)
        ->andReturn(new EsiResult(data: (object) ['ticker' => 'TCORP']));

    Artisan::call('generate:static-data');

    expect(File::exists("$resourcesPath/regions.json"))->toBeTrue();
    expect(File::exists("$resourcesPath/constellations.json"))->toBeTrue();
    expect(File::exists("$resourcesPath/solarsystems.json"))->toBeTrue();
    expect(File::exists("$resourcesPath/connections.json"))->toBeTrue();
    expect(File::exists("$resourcesPath/services.json"))->toBeTrue();
    expect(File::exists("$storagePath/sovereignty.json.gz"))->toBeTrue();

    $solarsystems = decodePlainJson("$resourcesPath/solarsystems.json");
    $sovereignties = decodeCompressedJson("$storagePath/sovereignty.json.gz");

    expect($solarsystems)->toHaveCount(1)
        ->and($solarsystems[0]['id'])->toBe(30000001)
        ->and($solarsystems[0]['has_stations'])->toBeTrue()
        ->and($solarsystems[0]['services'])->toBe([5, 27])
        ->and($sovereignties)->toHaveKey('30000001')
        ->and($sovereignties['30000001']['alliance']['ticker'])->toBe('TALL')
        ->and($sovereignties['30000001']['corporation']['ticker'])->toBe('TCORP');
});

it('can skip sovereignty generation', function () {
    $resourcesPath = resource_path('static');
    $storagePath = storage_path('app/static');

    Http::fake();
    File::delete("$storagePath/sovereignty.json.gz");

    $esi = $this->mock(Esi::class);
    $esi->shouldNotReceive('getSovereignty');
    $esi->shouldNotReceive('getNames');
    $esi->shouldNotReceive('getAlliance');
    $esi->shouldNotReceive('getCorporation');

    Artisan::call('generate:static-data --skip-sovereignty');

    expect(File::exists("$resourcesPath/regions.json"))->toBeTrue();
    expect(File::exists("$resourcesPath/constellations.json"))->toBeTrue();
    expect(File::exists("$resourcesPath/solarsystems.json"))->toBeTrue();
    expect(File::exists("$resourcesPath/connections.json"))->toBeTrue();
    expect(File::exists("$resourcesPath/services.json"))->toBeTrue();
    expect(File::exists("$storagePath/sovereignty.json.gz"))->toBeFalse();
});

it('serves the sovereignty file with cache headers', function () {
    $storagePath = storage_path('app/static');

    $user = User::factory()->create();
    $sovereigntyData = gzencode(json_encode(['123' => ['id' => 1]], JSON_THROW_ON_ERROR), 9);

    if ($sovereigntyData === false) {
        throw new RuntimeException('Failed to create compressed sovereignty fixture.');
    }

    File::put("$storagePath/sovereignty.json.gz", $sovereigntyData);

    actingAs($user);

    $response = $this->get(route('static.sovereignty'));

    $response->assertOk();
    $response->assertHeader('Cache-Control');
    $response->assertHeader('Content-Encoding', 'gzip');
    $response->assertHeader('Content-Type', 'application/json');

    expect((string) $response->headers->get('Cache-Control'))
        ->toContain('public')
        ->toContain('max-age=86400');
});

/**
 * @param  array<int, array<string, mixed>>  $rows
 */
function writeJsonl(string $path, array $rows): void
{
    $lines = array_map(static fn (array $row): string => json_encode($row, JSON_THROW_ON_ERROR), $rows);
    File::put($path, implode("\n", $lines)."\n");
}

function decodeCompressedJson(string $path): array
{
    if (! File::exists($path)) {
        throw new RuntimeException("File not found: $path");
    }

    $contents = file_get_contents($path);
    if (! is_string($contents)) {
        throw new RuntimeException("Unable to read file: $path");
    }

    $decoded = gzdecode($contents);

    if ($decoded === false) {
        return [];
    }

    return json_decode($decoded, true, flags: JSON_THROW_ON_ERROR);
}

function decodePlainJson(string $path): array
{
    if (! File::exists($path)) {
        throw new RuntimeException("File not found: $path");
    }

    $contents = file_get_contents($path);
    if (! is_string($contents)) {
        throw new RuntimeException("Unable to read file: $path");
    }

    return json_decode($contents, true, flags: JSON_THROW_ON_ERROR);
}
