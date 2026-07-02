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
use App\Events\Signatures\SignaturesChangedEvent;
use App\Models\Map;
use Illuminate\Support\Facades\Event;

it('broadcasts the signature counts once when a signature is stored', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30023001);

    Event::fake([SignaturesChangedEvent::class]);

    app(StoreSignatureAction::class)->handle($system, NewSignatureData::from(['signature_id' => 'ABC-123']));

    Event::assertDispatchedTimes(SignaturesChangedEvent::class, 1);
    Event::assertDispatched(SignaturesChangedEvent::class, function (SignaturesChangedEvent $event) use ($map, $system): bool {
        $payload = $event->broadcastWith();

        return $event->map_id === $map->id
            && $payload['map_solarsystem_id'] === $system->id
            && $payload['signature_counts'] === [
                'signatures_count' => 1,
                'wormhole_signatures_count' => 0,
                'uncategorized_signatures_count' => 1,
            ];
    });
});

it('broadcasts the signature counts once when a signature is updated', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30023002);
    $signature = $system->signatures()->create(['signature_id' => 'ABC-123']);

    Event::fake([SignaturesChangedEvent::class]);

    app(UpdateSignatureAction::class)->handle($signature, SignatureData::from(['signature_id' => 'XYZ-999']));

    Event::assertDispatchedTimes(SignaturesChangedEvent::class, 1);
    Event::assertDispatched(SignaturesChangedEvent::class, function (SignaturesChangedEvent $event) use ($system): bool {
        $payload = $event->broadcastWith();

        return $payload['map_solarsystem_id'] === $system->id
            && $payload['signature_counts']['signatures_count'] === 1;
    });
});

it('broadcasts the signature counts once when a signature is deleted', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30023003);
    $signature = $system->signatures()->create(['signature_id' => 'ABC-123']);

    Event::fake([SignaturesChangedEvent::class]);

    app(DeleteSignatureAction::class)->handle($signature);

    Event::assertDispatchedTimes(SignaturesChangedEvent::class, 1);
    Event::assertDispatched(SignaturesChangedEvent::class, function (SignaturesChangedEvent $event) use ($system): bool {
        $payload = $event->broadcastWith();

        return $payload['map_solarsystem_id'] === $system->id
            && $payload['signature_counts']['signatures_count'] === 0;
    });
});

it('broadcasts the signature counts once for a bulk delete', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30023004);
    $first = $system->signatures()->create(['signature_id' => 'AAA-111']);
    $second = $system->signatures()->create(['signature_id' => 'BBB-222']);

    Event::fake([SignaturesChangedEvent::class]);

    app(DeleteSignaturesAction::class)->handle($system, [$first->id, $second->id]);

    Event::assertDispatchedTimes(SignaturesChangedEvent::class, 1);
    Event::assertDispatched(SignaturesChangedEvent::class, function (SignaturesChangedEvent $event) use ($system): bool {
        $payload = $event->broadcastWith();

        return $payload['map_solarsystem_id'] === $system->id
            && $payload['signature_counts']['signatures_count'] === 0;
    });
});

it('broadcasts the signature counts once for a paste of multiple signatures', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30023005);

    Event::fake([SignaturesChangedEvent::class]);

    app(PasteSignaturesAction::class)->handle(SignaturesData::from([
        'map_solarsystem_id' => $system->id,
        'signatures' => [
            ['signature_id' => 'AAA-111'],
            ['signature_id' => 'BBB-222'],
            ['signature_id' => 'CCC-333'],
        ],
    ]));

    Event::assertDispatchedTimes(SignaturesChangedEvent::class, 1);
    Event::assertDispatched(SignaturesChangedEvent::class, function (SignaturesChangedEvent $event) use ($map, $system): bool {
        $payload = $event->broadcastWith();

        return $event->map_id === $map->id
            && $payload['map_solarsystem_id'] === $system->id
            && $payload['signature_counts']['signatures_count'] === 3;
    });
});
