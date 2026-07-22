<?php

declare(strict_types=1);

use App\Actions\MapConnections\CreateMapConnectionAction;
use App\Actions\MapSolarsystem\StoreMapSolarsystemAction;
use App\Jobs\MapAlerts\EvaluateMapAlertsJob;
use App\Models\Map;
use Illuminate\Support\Facades\Queue;

beforeEach(function () {
    Queue::fake();
});

it('queues an evaluation for the added system when a system joins the map', function () {
    $map = Map::factory()->create();
    $sid = makeSolarsystem(30009200);

    app(StoreMapSolarsystemAction::class)->handle($map, [
        'solarsystem_id' => $sid,
        'position_x' => 1,
        'position_y' => 1,
    ]);

    Queue::assertPushed(
        EvaluateMapAlertsJob::class,
        fn (EvaluateMapAlertsJob $job): bool => $job->map_solarsystem_id === $map->mapSolarsystems()->sole()->id,
    );
});

it('queues evaluations for both endpoints when a connection is created', function () {
    $map = Map::factory()->create();
    $from = placeMapSolarsystem($map, 30009202);
    $to = placeMapSolarsystem($map, 30009203, 200, 200);

    app(CreateMapConnectionAction::class)->handle([
        'from_map_solarsystem_id' => $from->id,
        'to_map_solarsystem_id' => $to->id,
    ]);

    Queue::assertPushed(
        EvaluateMapAlertsJob::class,
        fn (EvaluateMapAlertsJob $job): bool => $job->map_solarsystem_id === $from->id,
    );
    Queue::assertPushed(
        EvaluateMapAlertsJob::class,
        fn (EvaluateMapAlertsJob $job): bool => $job->map_solarsystem_id === $to->id,
    );
});
