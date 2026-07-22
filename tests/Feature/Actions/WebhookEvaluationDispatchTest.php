<?php

declare(strict_types=1);

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
