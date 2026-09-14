<?php

declare(strict_types=1);

use App\Actions\Signatures\PasteSignaturesAction;
use App\Data\NewSignatureData;
use App\Data\SignaturesData;
use App\Models\Map;

it('stores is_anomaly per row when pasting a mixed scan window', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30011020);

    app(PasteSignaturesAction::class)->handle(SignaturesData::from([
        'map_solarsystem_id' => $system->id,
        'signatures' => [
            ['signature_id' => 'AAA-111', 'is_anomaly' => true],
            ['signature_id' => 'BBB-222', 'is_anomaly' => false],
        ],
    ]));

    expect($system->signatures()->where('signature_id', 'AAA-111')->first()->is_anomaly)->toBeTrue()
        ->and($system->signatures()->where('signature_id', 'BBB-222')->first()->is_anomaly)->toBeFalse();
});

it('flips is_anomaly when the same signature is re-pasted under a different group', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30011021);

    app(PasteSignaturesAction::class)->handle(SignaturesData::from([
        'map_solarsystem_id' => $system->id,
        'signatures' => [
            ['signature_id' => 'AAA-111', 'is_anomaly' => true],
        ],
    ]));

    expect($system->signatures()->where('signature_id', 'AAA-111')->first()->is_anomaly)->toBeTrue();

    app(PasteSignaturesAction::class)->handle(SignaturesData::from([
        'map_solarsystem_id' => $system->id,
        'signatures' => [
            ['signature_id' => 'AAA-111', 'is_anomaly' => false],
        ],
    ]));

    expect($system->signatures()->where('signature_id', 'AAA-111')->first()->is_anomaly)->toBeFalse();
});

it('defaults a manually created signature (no scan group) to is_anomaly = false', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30011022);

    $signature = app(App\Actions\Signatures\StoreSignatureAction::class)->handle($system, NewSignatureData::from(['signature_id' => 'ABC-123']));

    expect($signature->fresh()->is_anomaly)->toBeFalse();
});

it('keeps an existing signature\'s is_anomaly flag when a paste payload omits the field', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30011023);

    app(PasteSignaturesAction::class)->handle(SignaturesData::from([
        'map_solarsystem_id' => $system->id,
        'signatures' => [
            ['signature_id' => 'AAA-111', 'is_anomaly' => true],
        ],
    ]));

    expect($system->signatures()->where('signature_id', 'AAA-111')->first()->is_anomaly)->toBeTrue();

    app(PasteSignaturesAction::class)->handle(SignaturesData::from([
        'map_solarsystem_id' => $system->id,
        'signatures' => [
            ['signature_id' => 'AAA-111'],
        ],
    ]));

    expect($system->signatures()->where('signature_id', 'AAA-111')->first()->is_anomaly)->toBeTrue();
});
