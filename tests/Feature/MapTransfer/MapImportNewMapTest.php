<?php

declare(strict_types=1);

use App\Actions\MapTransfer\ExportMapAction;
use App\Enums\MapSolarsystemStatus;
use App\Enums\Permission;
use App\Models\Character;
use App\Models\Corporation;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapConnection;
use App\Models\Signature;
use App\Models\User;
use Illuminate\Http\UploadedFile;

use function Pest\Laravel\actingAs;

function transferImporter(): User
{
    $user = User::factory()->has(Character::factory())->create();

    $user->forceFill(['preferred_character_id' => $user->characters()->value('id')])->save();

    return $user->refresh();
}

function transferNewMapFile(array $sections, string $map_name = 'Imported Map'): UploadedFile
{
    return UploadedFile::fake()->createWithContent('export.json', json_encode([
        'format' => 'wormholesystems-map-export',
        'version' => 1,
        'exported_at' => now()->toIso8601String(),
        'map_name' => $map_name,
        'sections' => $sections,
    ]));
}

it('creates a new map owned by the importer', function () {
    makeSolarsystem(31000601);
    $user = transferImporter();

    $file = transferNewMapFile([
        'settings' => [
            'name' => 'Imported Chain',
            'layout' => 'manual',
            'allow_layout_override' => true,
            'constant_width_enabled' => false,
            'bookmark_format_wormhole' => '{alias}',
            'bookmark_format_kspace' => '{name}',
            'bookmark_format_return' => 'return',
            'bookmark_alias_scheme' => 'numeric',
            'bookmark_ignored_alias' => null,
            'home_solarsystem_id' => 31000601,
            'rally_solarsystem_id' => null,
        ],
        'solarsystems' => [
            [
                'solarsystem_id' => 31000601,
                'alias' => 'Home',
                'position_x' => 120,
                'position_y' => 340,
                'pinned' => true,
                'status' => MapSolarsystemStatus::Friendly->value,
                'occupier_alias' => 'KRAB',
                'notes' => 'Imported staging notes.',
            ],
        ],
        'routes' => [
            'route_solarsystems' => [['solarsystem_id' => 31000601, 'is_pinned' => false]],
            'ignored_solarsystems' => [],
        ],
    ]);

    actingAs($user)
        ->post(route('maps.import.store'), [
            'file' => $file,
            'sections' => ['settings', 'solarsystems', 'routes'],
        ])
        ->assertRedirect()
        ->assertSessionDoesntHaveErrors();

    $map = Map::query()->where('name', 'Imported Chain')->firstOrFail();
    $owner = $map->mapOwner;

    expect($owner->accessible_id)->toBe($user->active_character->id)
        ->and($owner->accessible_type)->toBe(Character::class)
        ->and($map->share_token)->toBeNull()
        ->and($map->is_public)->toBeFalse()
        ->and($map->home_solarsystem_id)->toBe(31000601)
        ->and($map->mapSolarsystems()->count())->toBe(1)
        ->and($map->mapSolarsystems()->first()->details->notes)->toBe('Imported staging notes.')
        ->and($map->mapRouteSolarsystems()->pluck('solarsystem_id')->all())->toBe([31000601]);
});

it('respects the name override', function () {
    $user = transferImporter();

    actingAs($user)
        ->post(route('maps.import.store'), [
            'file' => transferNewMapFile(['solarsystems' => []]),
            'sections' => ['solarsystems'],
            'name' => 'My Copy',
        ])
        ->assertRedirect()
        ->assertSessionDoesntHaveErrors();

    expect(Map::query()->where('name', 'My Copy')->exists())->toBeTrue();
});

it('rejects connections without the solar systems section for a new map', function () {
    $user = transferImporter();

    actingAs($user)
        ->post(route('maps.import.store'), [
            'file' => transferNewMapFile([
                'connections' => [],
            ]),
            'sections' => ['connections'],
        ])
        ->assertRedirect()
        ->assertSessionHasErrors('file');

    expect(Map::query()->count())->toBe(0);
});

it('round-trips a full export into an equivalent new map', function () {
    $source = Map::factory()->create();
    $from = placeMapSolarsystem($source, 31000602);
    $to = placeMapSolarsystem($source, 31000603);
    $from->update(['alias' => 'Home']);
    $from->details->update(['notes' => 'Round trip notes.', 'status' => MapSolarsystemStatus::Friendly]);
    $wormhole = makeWormhole('H296');
    $connection = MapConnection::factory()->create([
        'map_id' => $source->id,
        'from_map_solarsystem_id' => $from->id,
        'to_map_solarsystem_id' => $to->id,
        'wormhole_id' => $wormhole->id,
    ]);
    Signature::factory()->create([
        'map_solarsystem_id' => $from->id,
        'signature_id' => 'RTA-001',
        'map_connection_id' => $connection->id,
        'wormhole_id' => $wormhole->id,
    ]);
    $corporation = Corporation::factory()->create(['name' => 'Krab Horizon']);
    MapAccess::factory()->for($source)->create([
        'accessible_type' => Corporation::class,
        'accessible_id' => $corporation->id,
        'permission' => Permission::Member,
        'is_owner' => false,
    ]);

    $sections = ['settings', 'access', 'solarsystems', 'connections', 'signatures', 'routes'];
    $payload = app(ExportMapAction::class)->handle($source->refresh(), $sections);
    $file = UploadedFile::fake()->createWithContent('export.json', json_encode($payload));

    $user = transferImporter();

    actingAs($user)
        ->post(route('maps.import.store'), [
            'file' => $file,
            'sections' => $sections,
            'name' => 'Round Trip Copy',
        ])
        ->assertRedirect()
        ->assertSessionDoesntHaveErrors();

    $copy = Map::query()->where('name', 'Round Trip Copy')->firstOrFail();
    $copied_from = $copy->mapSolarsystems()->where('solarsystem_id', 31000602)->first();
    $copied_signature = Signature::query()->where('map_solarsystem_id', $copied_from->id)->first();

    expect($copy->mapSolarsystems()->count())->toBe(2)
        ->and($copy->mapConnections()->count())->toBe(1)
        ->and($copied_from->alias)->toBe('Home')
        ->and($copied_from->details->notes)->toBe('Round trip notes.')
        ->and($copied_signature->signature_id)->toBe('RTA-001')
        ->and($copied_signature->map_connection_id)->toBe($copy->mapConnections()->value('id'))
        ->and($copy->mapAccessors()->where('is_owner', false)->count())->toBe(1)
        ->and($copy->mapAccessors()->where('is_owner', false)->first()->accessible_id)->toBe((int) $corporation->id)
        ->and($copy->mapOwner->accessible_id)->toBe($user->active_character->id)
        ->and($copy->share_token)->toBeNull();
});

function rallyImportSettings(array $rally): array
{
    return [
        'name' => 'Rally Chain',
        'layout' => 'manual',
        'allow_layout_override' => true,
        'constant_width_enabled' => false,
        'bookmark_format_wormhole' => '{alias}',
        'bookmark_format_kspace' => '{name}',
        'bookmark_format_return' => 'return',
        'bookmark_alias_scheme' => 'numeric',
        'bookmark_ignored_alias' => null,
        'home_solarsystem_id' => null,
        ...$rally,
    ];
}

it('imports rally points', function (array $rally, array $expected) {
    makeSolarsystem(31000601);
    makeSolarsystem(31000602);
    makeSolarsystem(31000603);

    actingAs(transferImporter())
        ->post(route('maps.import.store'), [
            'file' => transferNewMapFile(['settings' => rallyImportSettings($rally)]),
            'sections' => ['settings'],
        ])
        ->assertRedirect()
        ->assertSessionDoesntHaveErrors();

    expect(Map::query()->where('name', 'Rally Chain')->firstOrFail()->rally_solarsystem_ids)->toBe($expected);
})->with([
    'list' => [['rally_solarsystem_ids' => [31000601, 31000602]], [31000601, 31000602]],
    'legacy single id' => [['rally_solarsystem_id' => 31000601], [31000601]],
    'unknown ids dropped' => [['rally_solarsystem_ids' => [99999999, 31000602]], [31000602]],
    'capped to the newest' => [['rally_solarsystem_ids' => [31000601, 31000602, 31000603]], [31000602, 31000603]],
    'none' => [[], []],
]);

it('rejects malformed rally points', function () {
    actingAs(transferImporter())
        ->post(route('maps.import.store'), [
            'file' => transferNewMapFile(['settings' => rallyImportSettings(['rally_solarsystem_ids' => ['abc']])]),
            'sections' => ['settings'],
        ])
        ->assertSessionHasErrors('file');
});
