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
use App\Models\Map;
use App\Models\Signature;

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
