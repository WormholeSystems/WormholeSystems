<?php

declare(strict_types=1);

use App\Jobs\Webhooks\EvaluateMapWebhooksJob;
use App\Models\Map;
use App\Models\MapWebhook;
use Illuminate\Support\Facades\Http;

function runWebhookEval(Map $map, int $solarsystemId): void
{
    app()->call([new EvaluateMapWebhooksJob($map->id, $solarsystemId), 'handle']);
}

beforeEach(function () {
    Http::fake();
});

it('fires when the added system is within range of the target', function () {
    $sid = makeSolarsystem(30009001);
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create([
        'target_solarsystem_id' => $sid,
        'max_jumps' => 3,
    ]);

    runWebhookEval($map, $sid);

    Http::assertSentCount(1);

    expect($webhook->refresh()->last_fired_at)->not->toBeNull();
});

it('does not fire when the added system is out of range', function () {
    $origin = makeSolarsystem(30009002);
    $target = makeSolarsystem(30009003);

    $map = Map::factory()->create();
    MapWebhook::factory()->for($map)->create([
        'target_solarsystem_id' => $target,
        'max_jumps' => 3,
    ]);

    runWebhookEval($map, $origin);

    Http::assertNothingSent();
});

it('skips inactive webhooks', function () {
    $sid = makeSolarsystem(30009001);
    $map = Map::factory()->create();
    MapWebhook::factory()->for($map)->inactive()->create([
        'target_solarsystem_id' => $sid,
        'max_jumps' => 3,
    ]);

    runWebhookEval($map, $sid);

    Http::assertNothingSent();
});

it('fires across a real stargate hop', function () {
    /** @var array<int|string, int[]> $raw */
    $raw = json_decode(file_get_contents(resource_path('static/connections.json')), true);
    $origin = (int) array_key_first($raw);
    $target = (int) $raw[$origin][0];

    makeSolarsystem($origin);
    makeSolarsystem($target);

    $map = Map::factory()->create();
    MapWebhook::factory()->for($map)->create([
        'target_solarsystem_id' => $target,
        'max_jumps' => 1,
    ]);

    runWebhookEval($map, $origin);

    Http::assertSentCount(1);
});

it('colours the embed green for a highsec target', function () {
    $sid = makeSolarsystem(30009010, 0.9);
    $map = Map::factory()->create();
    MapWebhook::factory()->for($map)->create(['target_solarsystem_id' => $sid, 'max_jumps' => 1]);

    runWebhookEval($map, $sid);

    Http::assertSent(fn ($request): bool => $request->data()['embeds'][0]['color'] === 0x2ECC71);
});

it('colours the embed red for a nullsec target', function () {
    $sid = makeSolarsystem(30009011, -0.5);
    $map = Map::factory()->create();
    MapWebhook::factory()->for($map)->create(['target_solarsystem_id' => $sid, 'max_jumps' => 1]);

    runWebhookEval($map, $sid);

    Http::assertSent(fn ($request): bool => $request->data()['embeds'][0]['color'] === 0xE74C3C);
});

it('does not count wormhole connections, only k-space jumps', function () {
    // A J-space system has no stargates, so even though it is the added system it can
    // never be within gate range of a k-space target.
    $origin = makeSolarsystem(31000001);
    $target = makeSolarsystem(30009005);

    $map = Map::factory()->create();
    MapWebhook::factory()->for($map)->create([
        'target_solarsystem_id' => $target,
        'max_jumps' => 20,
    ]);

    runWebhookEval($map, $origin);

    Http::assertNothingSent();
});
