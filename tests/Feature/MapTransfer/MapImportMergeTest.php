<?php

declare(strict_types=1);

use App\Enums\MapSolarsystemStatus;
use App\Enums\Permission;
use App\Events\Maps\MapResyncEvent;
use App\Models\Character;
use App\Models\Corporation;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapConnection;
use App\Models\MapRouteSolarsystem;
use App\Models\MapSolarsystem;
use App\Models\MapSolarsystemDetails;
use App\Models\Signature;
use App\Models\User;
use Illuminate\Http\UploadedFile;
use Illuminate\Support\Facades\Event;

use function Pest\Laravel\actingAs;

function transferMergeUser(Map $map, Permission $permission): User
{
    $user = User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => $permission])->for($map)))
        ->create();

    $user->forceFill(['preferred_character_id' => $user->characters()->value('id')])->save();

    return $user->refresh();
}

function transferFile(array $sections, array $overrides = []): UploadedFile
{
    $payload = array_merge([
        'format' => 'wormholesystems-map-export',
        'version' => 1,
        'exported_at' => now()->toIso8601String(),
        'map_name' => 'Imported Map',
        'sections' => $sections,
    ], $overrides);

    return UploadedFile::fake()->createWithContent('export.json', json_encode($payload));
}

function transferSystemEntry(int $solarsystem_id, array $overrides = []): array
{
    return array_merge([
        'solarsystem_id' => $solarsystem_id,
        'alias' => null,
        'position_x' => 100,
        'position_y' => 100,
        'pinned' => false,
        'status' => MapSolarsystemStatus::Unknown->value,
        'occupier_alias' => null,
        'notes' => null,
    ], $overrides);
}

function transferConnectionEntry(int $from, int $to, array $overrides = []): array
{
    return array_merge([
        'from_solarsystem_id' => $from,
        'to_solarsystem_id' => $to,
        'wormhole' => null,
        'type' => 'wormhole',
        'mass_status' => 'fresh',
        'ship_size' => 'large',
        'lifetime' => 'healthy',
        'lifetime_updated_at' => null,
        'connected_at' => now()->toIso8601String(),
        'preserve_mass' => false,
    ], $overrides);
}

function transferSignatureEntry(int $solarsystem_id, ?string $signature_id, array $overrides = []): array
{
    return array_merge([
        'solarsystem_id' => $solarsystem_id,
        'signature_id' => $signature_id,
        'category' => null,
        'type_name' => null,
        'raw_type_name' => null,
        'wormhole' => null,
        'connection_index' => null,
        'mass_status' => null,
        'ship_size' => null,
        'lifetime' => 'healthy',
        'lifetime_updated_at' => null,
    ], $overrides);
}

it('merges systems, connections, signatures, routes, and access into the map', function () {
    Event::fake([MapResyncEvent::class]);

    $map = Map::factory()->create();
    makeSolarsystem(31000501);
    makeSolarsystem(31000502);
    makeSolarsystem(30000142);
    makeWormhole('H296');

    $file = transferFile([
        'solarsystems' => [
            transferSystemEntry(31000501, ['alias' => 'Home', 'notes' => 'Staging.', 'status' => 'friendly']),
            transferSystemEntry(31000502, ['position_x' => 300, 'position_y' => 200]),
        ],
        'connections' => [
            transferConnectionEntry(31000501, 31000502, ['wormhole' => 'H296']),
        ],
        'signatures' => [
            transferSignatureEntry(31000501, 'ABC-123', ['wormhole' => 'H296', 'connection_index' => 0]),
        ],
        'routes' => [
            'route_solarsystems' => [['solarsystem_id' => 30000142, 'is_pinned' => true]],
            'ignored_solarsystems' => [['solarsystem_id' => 31000502]],
        ],
        'access' => [
            ['entity_type' => 'corporation', 'entity_id' => 98000001, 'entity_name' => 'Krab Horizon', 'permission' => 'member', 'expires_at' => null],
        ],
    ]);

    actingAs(transferMergeUser($map, Permission::Manager))
        ->post(route('maps.settings.transfer.import', $map), [
            'file' => $file,
            'sections' => ['solarsystems', 'connections', 'signatures', 'routes', 'access'],
        ])
        ->assertRedirect()
        ->assertSessionDoesntHaveErrors();

    $from = $map->mapSolarsystems()->where('solarsystem_id', 31000501)->first();
    $to = $map->mapSolarsystems()->where('solarsystem_id', 31000502)->first();
    $connection = $map->mapConnections()->first();
    $signature = Signature::query()->where('map_solarsystem_id', $from->id)->first();

    expect($from->alias)->toBe('Home')
        ->and($from->details->notes)->toBe('Staging.')
        ->and($from->details->status)->toBe(MapSolarsystemStatus::Friendly)
        ->and($connection->from_map_solarsystem_id)->toBe($from->id)
        ->and($connection->to_map_solarsystem_id)->toBe($to->id)
        ->and($connection->wormhole->name)->toBe('H296')
        ->and($signature->signature_id)->toBe('ABC-123')
        ->and($signature->map_connection_id)->toBe($connection->id)
        ->and($map->mapRouteSolarsystems()->pluck('solarsystem_id')->all())->toBe([30000142])
        ->and($map->mapIgnoredSolarsystems()->pluck('solarsystem_id')->all())->toBe([31000502])
        ->and(Corporation::query()->find(98000001)->name)->toBe('Krab Horizon')
        ->and(MapAccess::query()->where('map_id', $map->id)->where('accessible_id', 98000001)->first()->permission)->toBe(Permission::Member);

    Event::assertDispatched(MapResyncEvent::class);
});

it('merges routes into the shared watchlist without touching personal rows', function () {
    Event::fake([MapResyncEvent::class]);

    $map = Map::factory()->create();
    makeSolarsystem(30000142);
    $personal = MapRouteSolarsystem::factory()->personal()->create([
        'map_id' => $map->id,
        'solarsystem_id' => 30000142,
        'is_pinned' => false,
    ]);

    actingAs(transferMergeUser($map, Permission::Manager))
        ->post(route('maps.settings.transfer.import', $map), [
            'file' => transferFile([
                'routes' => [
                    'route_solarsystems' => [['solarsystem_id' => 30000142, 'is_pinned' => true]],
                    'ignored_solarsystems' => [],
                ],
            ]),
            'sections' => ['routes'],
        ])
        ->assertRedirect()
        ->assertSessionDoesntHaveErrors();

    $shared = $map->mapRouteSolarsystems()->sole();

    expect($shared->id)->not->toBe($personal->id)
        ->and($shared->is_pinned)->toBeTrue()
        ->and($personal->fresh()->is_pinned)->toBeFalse()
        ->and($personal->fresh()->user_id)->toBe($personal->user_id);
});

it('denies members', function () {
    $map = Map::factory()->create();

    actingAs(transferMergeUser($map, Permission::Member))
        ->post(route('maps.settings.transfer.import', $map), [
            'file' => transferFile(['solarsystems' => []]),
            'sections' => ['solarsystems'],
        ])
        ->assertForbidden();
});

it('updates existing systems and signatures in place', function () {
    $map = Map::factory()->create();
    $placement = placeMapSolarsystem($map, 31000503);
    $placement->details->update(['notes' => 'Stale intel.', 'status' => MapSolarsystemStatus::Hostile]);
    Signature::factory()->create([
        'map_solarsystem_id' => $placement->id,
        'signature_id' => 'DEF-456',
        'raw_type_name' => 'Old Site',
    ]);

    $file = transferFile([
        'solarsystems' => [
            transferSystemEntry(31000503, ['position_x' => 500, 'position_y' => 600, 'notes' => 'Fresh intel.', 'status' => 'friendly']),
        ],
        'signatures' => [
            transferSignatureEntry(31000503, 'DEF-456', ['raw_type_name' => 'New Site']),
        ],
    ]);

    actingAs(transferMergeUser($map, Permission::Manager))
        ->post(route('maps.settings.transfer.import', $map), [
            'file' => $file,
            'sections' => ['solarsystems', 'signatures'],
        ])
        ->assertRedirect()
        ->assertSessionDoesntHaveErrors();

    $placement->refresh();

    expect($placement->position_x)->toBe(500)
        ->and($placement->position_y)->toBe(600)
        ->and($placement->details->notes)->toBe('Fresh intel.')
        ->and($placement->details->status)->toBe(MapSolarsystemStatus::Friendly)
        ->and($placement->signatures()->count())->toBe(1)
        ->and($placement->signatures()->first()->raw_type_name)->toBe('New Site');
});

it('skips a duplicate connection but links signatures to the existing edge', function () {
    $map = Map::factory()->create();
    $from = placeMapSolarsystem($map, 31000504);
    $to = placeMapSolarsystem($map, 31000505);
    $existing = MapConnection::factory()->create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $to->id,
        'to_map_solarsystem_id' => $from->id,
    ]);

    $file = transferFile([
        'solarsystems' => [
            transferSystemEntry(31000504),
            transferSystemEntry(31000505),
        ],
        'connections' => [
            transferConnectionEntry(31000504, 31000505),
        ],
        'signatures' => [
            transferSignatureEntry(31000504, 'GHI-789', ['connection_index' => 0]),
        ],
    ]);

    actingAs(transferMergeUser($map, Permission::Manager))
        ->post(route('maps.settings.transfer.import', $map), [
            'file' => $file,
            'sections' => ['solarsystems', 'connections', 'signatures'],
        ])
        ->assertRedirect()
        ->assertSessionDoesntHaveErrors();

    expect($map->mapConnections()->count())->toBe(1)
        ->and(Signature::query()->where('signature_id', 'GHI-789')->first()->map_connection_id)->toBe($existing->id);
});

it('never touches the owner access row', function () {
    $map = Map::factory()->create();
    $owner_character = Character::factory()->create();
    MapAccess::factory()->for($map)->create([
        'accessible_type' => Character::class,
        'accessible_id' => $owner_character->id,
        'permission' => Permission::Manager,
        'is_owner' => true,
    ]);

    $file = transferFile([
        'access' => [
            ['entity_type' => 'character', 'entity_id' => $owner_character->id, 'entity_name' => $owner_character->name, 'permission' => 'viewer', 'expires_at' => null],
        ],
    ]);

    actingAs(transferMergeUser($map, Permission::Manager))
        ->post(route('maps.settings.transfer.import', $map), [
            'file' => $file,
            'sections' => ['access'],
        ])
        ->assertRedirect()
        ->assertSessionDoesntHaveErrors();

    $owner_access = MapAccess::query()
        ->where('map_id', $map->id)
        ->where('accessible_id', $owner_character->id)
        ->first();

    expect($owner_access->is_owner)->toBeTruthy()
        ->and($owner_access->permission)->toBe(Permission::Manager);
});

it('nulls unknown wormhole and signature type names but keeps the rows', function () {
    $map = Map::factory()->create();
    placeMapSolarsystem($map, 31000506);
    makeSolarsystem(31000507);

    $file = transferFile([
        'solarsystems' => [transferSystemEntry(31000507)],
        'connections' => [
            transferConnectionEntry(31000506, 31000507, ['wormhole' => 'Z999']),
        ],
        'signatures' => [
            transferSignatureEntry(31000506, 'JKL-012', ['type_name' => 'Unknown Site', 'raw_type_name' => 'Unknown Site']),
        ],
    ]);

    actingAs(transferMergeUser($map, Permission::Manager))
        ->post(route('maps.settings.transfer.import', $map), [
            'file' => $file,
            'sections' => ['solarsystems', 'connections', 'signatures'],
        ])
        ->assertRedirect()
        ->assertSessionDoesntHaveErrors();

    $connection = $map->mapConnections()->first();
    $signature = Signature::query()->where('signature_id', 'JKL-012')->first();

    expect($connection->wormhole_id)->toBeNull()
        ->and($signature->signature_type_id)->toBeNull()
        ->and($signature->raw_type_name)->toBe('Unknown Site');
});

it('skips connections whose endpoints are missing', function () {
    $map = Map::factory()->create();
    placeMapSolarsystem($map, 31000508);

    $file = transferFile([
        'connections' => [
            transferConnectionEntry(31000508, 31000999),
        ],
    ]);

    actingAs(transferMergeUser($map, Permission::Manager))
        ->post(route('maps.settings.transfer.import', $map), [
            'file' => $file,
            'sections' => ['connections'],
        ])
        ->assertRedirect()
        ->assertSessionDoesntHaveErrors();

    expect($map->mapConnections()->count())->toBe(0);
});

it('imports signatures without a scanned id and does not duplicate them on re-import', function () {
    $map = Map::factory()->create();
    placeMapSolarsystem($map, 31000509);
    makeSolarsystem(31000510);

    $file_sections = [
        'solarsystems' => [transferSystemEntry(31000510)],
        'connections' => [transferConnectionEntry(31000509, 31000510)],
        'signatures' => [
            transferSignatureEntry(31000509, null, ['connection_index' => 0]),
        ],
    ];

    $user = transferMergeUser($map, Permission::Manager);

    foreach (range(1, 2) as $attempt) {
        actingAs($user)
            ->post(route('maps.settings.transfer.import', $map), [
                'file' => transferFile($file_sections),
                'sections' => ['solarsystems', 'connections', 'signatures'],
            ])
            ->assertRedirect()
            ->assertSessionDoesntHaveErrors();
    }

    $signatures = Signature::query()->whereNull('signature_id')->get();

    expect($signatures)->toHaveCount(1)
        ->and($signatures->first()->map_connection_id)->toBe($map->mapConnections()->value('id'));
});

it('rejects files that are not valid exports', function (UploadedFile $file) {
    $map = Map::factory()->create();

    actingAs(transferMergeUser($map, Permission::Manager))
        ->from(route('maps.settings.transfer.show', $map))
        ->post(route('maps.settings.transfer.import', $map), [
            'file' => $file,
            'sections' => ['solarsystems'],
        ])
        ->assertRedirect(route('maps.settings.transfer.show', $map))
        ->assertSessionHasErrors('file');

    expect(MapSolarsystem::query()->count())->toBe(0)
        ->and(MapSolarsystemDetails::query()->count())->toBe(0);
})->with([
    'not JSON' => fn () => UploadedFile::fake()->createWithContent('export.json', 'not json at all'),
    'wrong format' => fn () => UploadedFile::fake()->createWithContent('export.json', json_encode(['format' => 'other-tool', 'version' => 1, 'sections' => []])),
    'newer version' => fn () => UploadedFile::fake()->createWithContent('export.json', json_encode(['format' => 'wormholesystems-map-export', 'version' => 2, 'map_name' => 'X', 'sections' => ['solarsystems' => []]])),
    'missing section' => fn () => transferFile(['settings' => []]),
    'invalid structure' => fn () => transferFile(['solarsystems' => [['solarsystem_id' => 'not-a-number']]]),
]);
