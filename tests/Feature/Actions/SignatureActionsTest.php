<?php

declare(strict_types=1);

use App\Actions\DeleteSignaturesAction;
use App\Actions\Signatures\DeleteSignatureAction;
use App\Actions\Signatures\PasteSignaturesAction;
use App\Actions\Signatures\StoreSignatureAction;
use App\Actions\Signatures\UpdateSignatureAction;
use App\Data\NewSignatureData;
use App\Data\SignatureData;
use App\Data\SignaturesData;
use App\Enums\ShipSize;
use App\Models\Map;
use App\Models\MapConnection;
use App\Models\Signature;
use App\Models\Wormhole;

it('stores a signature on a system', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30011001);

    $signature = app(StoreSignatureAction::class)->handle($system, NewSignatureData::from(['signature_id' => 'ABC-123']));

    expect($signature->signature_id)->toBe('ABC-123')
        ->and($signature->map_solarsystem_id)->toBe($system->id);
});

it('updates a signature', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30011002);
    $signature = $system->signatures()->create(['signature_id' => 'ABC-123']);

    app(UpdateSignatureAction::class)->handle($signature, SignatureData::from(['signature_id' => 'XYZ-999']));

    expect($signature->fresh()->signature_id)->toBe('XYZ-999');
});

it('deletes a signature', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30011003);
    $signature = $system->signatures()->create(['signature_id' => 'ABC-123']);

    app(DeleteSignatureAction::class)->handle($signature);

    expect(Signature::find($signature->id))->toBeNull();
});

it('deletes multiple signatures from a system', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30011004);
    $first = $system->signatures()->create(['signature_id' => 'AAA-111']);
    $second = $system->signatures()->create(['signature_id' => 'BBB-222']);

    app(DeleteSignaturesAction::class)->handle($system, [$first->id, $second->id]);

    expect($system->signatures()->count())->toBe(0);
});

it('pastes new signatures onto a system', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30011005);

    app(PasteSignaturesAction::class)->handle(SignaturesData::from([
        'map_solarsystem_id' => $system->id,
        'signatures' => [
            ['signature_id' => 'AAA-111'],
            ['signature_id' => 'BBB-222'],
        ],
    ]));

    expect($system->signatures()->count())->toBe(2);
});

it('syncs the connection ship size from the signature wormhole type', function () {
    $map = Map::factory()->create();
    $origin = placeMapSolarsystem($map, 30011010);
    $target = placeMapSolarsystem($map, 30011011, 300, 300);
    $connection = MapConnection::create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $origin->id,
        'to_map_solarsystem_id' => $target->id,
        'ship_size' => 'large',
        'lifetime' => 'healthy',
        'mass_status' => 'fresh',
    ]);
    $wormhole = Wormhole::create([
        'name' => 'H296',
        'total_mass' => 3_300_000_000,
        'maximum_jump_mass' => 2_000_000_000,
        'maximum_lifetime' => 86_400,
        'leads_to' => 'c5',
    ]);
    $signature = $origin->signatures()->create([
        'signature_id' => 'ABC-123',
        'map_connection_id' => $connection->id,
        'wormhole_id' => $wormhole->id,
        'lifetime' => 'healthy',
    ]);

    app(UpdateSignatureAction::class)->handle($signature, SignatureData::from(['signature_id' => 'ABC-123']));

    expect($connection->fresh()->ship_size)->toBe(ShipSize::ExtraLarge);
});

it('syncs the connection ship size when pasting over a typed connected signature', function () {
    $map = Map::factory()->create();
    $origin = placeMapSolarsystem($map, 30011012);
    $target = placeMapSolarsystem($map, 30011013, 300, 300);
    $connection = MapConnection::create([
        'map_id' => $map->id,
        'from_map_solarsystem_id' => $origin->id,
        'to_map_solarsystem_id' => $target->id,
        'ship_size' => 'large',
        'lifetime' => 'healthy',
        'mass_status' => 'fresh',
    ]);
    $wormhole = Wormhole::create([
        'name' => 'X877',
        'total_mass' => 2_000_000_000,
        'maximum_jump_mass' => 375_000_000,
        'maximum_lifetime' => 57_600,
        'leads_to' => 'c4',
    ]);
    $signature_type = App\Models\SignatureType::query()->where('signature', 'X877')->firstOrFail();
    $origin->signatures()->create([
        'signature_id' => 'AAA-111',
        'map_connection_id' => $connection->id,
        'signature_type_id' => $signature_type->id,
        'wormhole_id' => $wormhole->id,
        'lifetime' => 'healthy',
    ]);
    $connection->update(['ship_size' => 'xlarge']);

    app(PasteSignaturesAction::class)->handle(SignaturesData::from([
        'map_solarsystem_id' => $origin->id,
        'signatures' => [
            ['signature_id' => 'AAA-111'],
        ],
    ]));

    expect($connection->fresh()->ship_size)->toBe(ShipSize::Large);
});
