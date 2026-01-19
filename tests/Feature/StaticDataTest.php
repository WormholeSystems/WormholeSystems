<?php

declare(strict_types=1);

use App\Models\Constellation;
use App\Models\Region;
use App\Models\Solarsystem;
use App\Models\Sovereignty;
use App\Models\User;
use Illuminate\Support\Facades\Artisan;
use Illuminate\Support\Facades\File;

use function Pest\Laravel\actingAs;

beforeEach(function () {
    $this->resourcesPath = resource_path('static');
    $this->storagePath = storage_path('app/static');

    File::ensureDirectoryExists($this->resourcesPath);
    File::ensureDirectoryExists($this->storagePath);
});

afterEach(function () {
    File::delete([
        "{$this->resourcesPath}/regions.json",
        "{$this->resourcesPath}/constellations.json",
        "{$this->resourcesPath}/solarsystems.json",
        "{$this->resourcesPath}/connections.json",
        "{$this->storagePath}/sovereignty.json.gz",
    ]);
});

it('generates static data files', function () {
    $region = Region::query()->create([
        'id' => 10000001,
        'name' => 'Test Region',
        'type' => 'kspace',
    ]);

    $constellation = Constellation::query()->create([
        'id' => 20000001,
        'name' => 'Test Constellation',
        'type' => 'kspace',
        'region_id' => $region->id,
    ]);

    $solarsystem = Solarsystem::query()->create([
        'id' => 30000001,
        'name' => 'Test System',
        'constellation_id' => $constellation->id,
        'region_id' => $region->id,
        'security' => 0.5,
        'pos_x' => 0.0,
        'pos_y' => 0.0,
        'pos_z' => 0.0,
        'type' => 'eve',
    ]);

    Sovereignty::query()->create([
        'solarsystem_id' => $solarsystem->id,
    ]);

    Artisan::call('generate:static-data');

    expect(File::exists("{$this->resourcesPath}/regions.json"))->toBeTrue();
    expect(File::exists("{$this->resourcesPath}/constellations.json"))->toBeTrue();
    expect(File::exists("{$this->resourcesPath}/solarsystems.json"))->toBeTrue();
    expect(File::exists("{$this->resourcesPath}/connections.json"))->toBeTrue();
    expect(File::exists("{$this->storagePath}/sovereignty.json.gz"))->toBeTrue();

    $solarsystems = decodePlainJson("{$this->resourcesPath}/solarsystems.json");
    $sovereignties = decodeCompressedJson("{$this->storagePath}/sovereignty.json.gz");

    expect($solarsystems)->toHaveCount(1)
        ->and($solarsystems[0]['id'])->toBe($solarsystem->id)
        ->and($sovereignties)->toHaveKey((string) $solarsystem->id);
});

it('serves the sovereignty file with cache headers', function () {
    $user = User::factory()->create();
    $sovereigntyData = gzencode(json_encode(['123' => ['id' => 1]], JSON_THROW_ON_ERROR), 9);

    if ($sovereigntyData === false) {
        throw new RuntimeException('Failed to create compressed sovereignty fixture.');
    }

    File::put("{$this->storagePath}/sovereignty.json.gz", $sovereigntyData);

    actingAs($user);

    $response = $this->get(route('static.sovereignty'));

    $response->assertOk();
    $response->assertHeader('Cache-Control', 'public, max-age=86400');
    $response->assertHeader('Content-Encoding', 'gzip');
    $response->assertHeader('Content-Type', 'application/json');
});

function decodeCompressedJson(string $path): array
{
    $contents = File::get($path);
    $decoded = gzdecode($contents);

    if ($decoded === false) {
        return [];
    }

    return json_decode($decoded, true, flags: JSON_THROW_ON_ERROR);
}

function decodePlainJson(string $path): array
{
    return json_decode(File::get($path), true, flags: JSON_THROW_ON_ERROR);
}
