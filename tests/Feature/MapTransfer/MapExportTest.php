<?php

declare(strict_types=1);

use App\Enums\Permission;
use App\Models\Character;
use App\Models\Corporation;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapConnection;
use App\Models\MapIgnoredSolarsystem;
use App\Models\MapRouteSolarsystem;
use App\Models\MapSolarsystemDetails;
use App\Models\Signature;
use App\Models\SignatureCategory;
use App\Models\SignatureType;
use App\Models\User;

use function Pest\Laravel\actingAs;

function transferExportUser(Map $map, Permission $permission): User
{
    $user = User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => $permission])->for($map)))
        ->create();

    $user->forceFill(['preferred_character_id' => $user->characters()->value('id')])->save();

    return $user->refresh();
}

function transferExportPayload(Map $map, array $sections): array
{
    $response = actingAs(transferExportUser($map, Permission::Manager))
        ->get(route('maps.settings.transfer.export', ['map' => $map, 'sections' => $sections]))
        ->assertSuccessful();

    return json_decode($response->streamedContent(), true);
}

it('shows the transfer page with section counts for managers', function () {
    $map = Map::factory()->create();
    MapAccess::factory()->for($map)->create([
        'accessible_type' => Character::class,
        'accessible_id' => Character::factory()->create()->id,
        'permission' => Permission::Manager,
        'is_owner' => true,
    ]);
    $from = placeMapSolarsystem($map, 31000406);
    $to = placeMapSolarsystem($map, 31000407);
    MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $from->id,
        'to_map_solarsystem_id' => $to->id,
    ]);
    Signature::factory()->create(['map_solarsystem_id' => $from->id]);

    actingAs(transferExportUser($map, Permission::Manager))
        ->get(route('maps.settings.transfer.show', $map))
        ->assertSuccessful()
        ->assertInertia(
            fn ($page) => $page
                ->component('maps/settings/ShowTransfer')
                ->where('counts.solarsystems', 2)
                ->where('counts.connections', 1)
                ->where('counts.signatures', 1)
                ->where('counts.access', 1)
                ->where('counts.routes', 0),
        );
});

it('lets a manager download a versioned export', function () {
    $map = Map::factory()->create(['share_token' => 'secret-token']);
    placeMapSolarsystem($map, 31000401);

    $response = actingAs(transferExportUser($map, Permission::Manager))
        ->get(route('maps.settings.transfer.export', ['map' => $map, 'sections' => ['settings', 'solarsystems']]))
        ->assertSuccessful()
        ->assertDownload();

    $payload = json_decode($response->streamedContent(), true);

    expect($payload['format'])->toBe('wormholesystems-map-export')
        ->and($payload['version'])->toBe(1)
        ->and($payload['exported_at'])->not->toBeNull()
        ->and($payload['map_name'])->toBe($map->name)
        ->and(json_encode($payload))->not->toContain('secret-token');
});

it('denies members and guests', function () {
    $map = Map::factory()->create();

    $this->get(route('maps.settings.transfer.export', ['map' => $map, 'sections' => ['settings']]))
        ->assertRedirect();

    actingAs(transferExportUser($map, Permission::Member))
        ->get(route('maps.settings.transfer.export', ['map' => $map, 'sections' => ['settings']]))
        ->assertForbidden();
});

it('includes only the selected sections', function () {
    $map = Map::factory()->create();
    placeMapSolarsystem($map, 31000402);

    $payload = transferExportPayload($map, ['solarsystems']);

    expect($payload['sections'])->toHaveKey('solarsystems')
        ->and($payload['sections'])->not->toHaveKeys(['settings', 'access', 'connections', 'signatures', 'routes']);
});

it('exports access entries without the owner row', function () {
    $map = Map::factory()->create();
    $corporation = Corporation::factory()->create(['name' => 'Krab Horizon']);
    MapAccess::factory()->for($map)->create([
        'accessible_type' => Corporation::class,
        'accessible_id' => $corporation->id,
        'permission' => Permission::Member,
        'is_owner' => false,
    ]);
    $owner_character = Character::factory()->create();
    MapAccess::factory()->for($map)->create([
        'accessible_type' => Character::class,
        'accessible_id' => $owner_character->id,
        'permission' => Permission::Manager,
        'is_owner' => true,
    ]);

    $payload = transferExportPayload($map, ['access']);

    expect($payload['sections']['access'])->toHaveCount(2)
        ->and(collect($payload['sections']['access'])->pluck('entity_id'))->not->toContain($owner_character->id)
        ->and(collect($payload['sections']['access'])->firstWhere('entity_type', 'corporation'))
        ->toMatchArray(['entity_id' => $corporation->id, 'entity_name' => 'Krab Horizon', 'permission' => 'member']);
});

it('exports connections by wormhole name and signatures by connection index', function () {
    $map = Map::factory()->create();
    $from = placeMapSolarsystem($map, 31000403);
    $to = placeMapSolarsystem($map, 31000404);
    $wormhole = makeWormhole('H296');
    $connection = MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $from->id,
        'to_map_solarsystem_id' => $to->id,
        'wormhole_id' => $wormhole->id,
    ]);
    $category = SignatureCategory::query()->where('code', 'wormhole')->firstOrFail();
    $type = SignatureType::query()->firstOrCreate(['name' => 'H296 - C5', 'signature_category_id' => $category->id]);
    Signature::factory()->create([
        'map_solarsystem_id' => $from->id,
        'signature_id' => 'ABC-123',
        'map_connection_id' => $connection->id,
        'wormhole_id' => $wormhole->id,
        'signature_category_id' => $category->id,
        'signature_type_id' => $type->id,
    ]);

    $payload = transferExportPayload($map, ['connections', 'signatures']);

    expect($payload['sections']['connections'])->toHaveCount(1)
        ->and($payload['sections']['connections'][0])->toMatchArray([
            'from_solarsystem_id' => 31000403,
            'to_solarsystem_id' => 31000404,
            'wormhole' => 'H296',
        ])
        ->and($payload['sections']['signatures'][0])->toMatchArray([
            'solarsystem_id' => 31000403,
            'signature_id' => 'ABC-123',
            'category' => 'wormhole',
            'type_name' => 'H296 - C5',
            'wormhole' => 'H296',
            'connection_index' => 0,
        ]);
});

it('exports details-only systems with null positions', function () {
    $map = Map::factory()->create();
    makeSolarsystem(31000405);
    MapSolarsystemDetails::factory()->create([
        'map_id' => $map->id,
        'solarsystem_id' => 31000405,
        'notes' => 'Old staging intel.',
    ]);

    $payload = transferExportPayload($map, ['solarsystems']);

    expect($payload['sections']['solarsystems'])->toHaveCount(1)
        ->and($payload['sections']['solarsystems'][0])->toMatchArray([
            'solarsystem_id' => 31000405,
            'position_x' => null,
            'position_y' => null,
            'notes' => 'Old staging intel.',
        ]);
});

it('exports routes and ignored systems', function () {
    $map = Map::factory()->create();
    makeSolarsystem(30000142);
    makeSolarsystem(30002187);
    MapRouteSolarsystem::factory()->create(['map_id' => $map->id, 'solarsystem_id' => 30000142, 'is_pinned' => true]);
    MapIgnoredSolarsystem::factory()->create(['map_id' => $map->id, 'solarsystem_id' => 30002187]);

    $payload = transferExportPayload($map, ['routes']);

    expect($payload['sections']['routes']['route_solarsystems'])->toBe([['solarsystem_id' => 30000142, 'is_pinned' => true]])
        ->and($payload['sections']['routes']['ignored_solarsystems'])->toBe([['solarsystem_id' => 30002187]]);
});

it('exports a connection whose ship size nobody has set', function () {
    $map = Map::factory()->create();
    $from = placeMapSolarsystem($map, 31000420);
    $to = placeMapSolarsystem($map, 31000421);
    MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $from->id,
        'to_map_solarsystem_id' => $to->id,
        'ship_size' => null,
    ]);

    $payload = transferExportPayload($map, ['connections']);

    expect($payload['sections']['connections'][0]['ship_size'])->toBeNull();
});
