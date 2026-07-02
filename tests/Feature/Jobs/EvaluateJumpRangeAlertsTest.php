<?php

declare(strict_types=1);

use App\Enums\JumpShipType;
use App\Jobs\Webhooks\EvaluateMapWebhooksJob;
use App\Models\Map;
use App\Models\MapAlert;
use App\Models\MapWebhook;
use App\Services\JumpRange\JumpRangeCalculator;
use Illuminate\Support\Facades\Http;

function runJumpRangeEval(Map $map, int $solarsystemId): void
{
    app()->call([new EvaluateMapWebhooksJob($map->id, $solarsystemId), 'handle']);
}

/**
 * A jump-range alert on the map, delivering to a fresh webhook destination.
 *
 * @param  array<string, mixed>  $attributes
 */
function jumpRangeAlert(Map $map, array $attributes = []): MapAlert
{
    $webhook = MapWebhook::factory()->for($map)->create();

    return MapAlert::factory()->jumpRange()->create(array_merge([
        'map_id' => $map->id,
        'map_webhook_id' => $webhook->id,
    ], $attributes));
}

function lightyears(float $ly): float
{
    return $ly * JumpRangeCalculator::METERS_PER_LIGHTYEAR;
}

beforeEach(function () {
    Http::fake();
});

it('fires when a k-space exit is added within jump range of the staging system', function () {
    $staging = makeSolarsystem(30009101, -0.3, 'eve');
    $exit = makeSolarsystem(30009102, -0.1, 'eve', lightyears(6.0));

    $map = Map::factory()->create();
    $alert = jumpRangeAlert($map, ['target_solarsystem_id' => $staging]);

    runJumpRangeEval($map, $exit);

    Http::assertSentCount(1);
    Http::assertSent(fn ($request): bool => str_contains(json_encode($request->data()['embeds'][0]['fields']), '6.00 ly'));

    expect($alert->refresh()->last_fired_at)->not->toBeNull();
});

it('does not fire when the exit is out of jump range', function () {
    $staging = makeSolarsystem(30009103, -0.3, 'eve');
    $exit = makeSolarsystem(30009104, -0.1, 'eve', lightyears(8.0));

    $map = Map::factory()->create();
    jumpRangeAlert($map, ['target_solarsystem_id' => $staging]);

    runJumpRangeEval($map, $exit);

    Http::assertNothingSent();
});

it('respects the ship class and JDC level for the range', function () {
    $staging = makeSolarsystem(30009105, -0.3, 'eve');
    $exit = makeSolarsystem(30009106, -0.1, 'eve', lightyears(6.5));

    $map = Map::factory()->create();
    jumpRangeAlert($map, [
        'target_solarsystem_id' => $staging,
        'ship_type' => JumpShipType::Supercarrier,
        'jdc_level' => 5,
    ]);

    runJumpRangeEval($map, $exit);

    Http::assertNothingSent();
});

it('skips a high-sec exit by default', function () {
    $staging = makeSolarsystem(30009107, -0.3, 'eve');
    $exit = makeSolarsystem(30009108, 0.9, 'eve', lightyears(2.0));

    $map = Map::factory()->create();
    jumpRangeAlert($map, ['target_solarsystem_id' => $staging]);

    runJumpRangeEval($map, $exit);

    Http::assertNothingSent();
});

it('fires for a high-sec exit when include_highsec is enabled', function () {
    $staging = makeSolarsystem(30009109, -0.3, 'eve');
    $exit = makeSolarsystem(30009110, 0.9, 'eve', lightyears(2.0));

    $map = Map::factory()->create();
    jumpRangeAlert($map, ['target_solarsystem_id' => $staging, 'include_highsec' => true]);

    runJumpRangeEval($map, $exit);

    Http::assertSentCount(1);
});

it('never fires for a wormhole-space system', function () {
    $staging = makeSolarsystem(30009111, -0.3, 'eve');
    $exit = makeSolarsystem(31009112, -1.0, 'wh');

    $map = Map::factory()->create();
    jumpRangeAlert($map, ['target_solarsystem_id' => $staging]);

    runJumpRangeEval($map, $exit);

    Http::assertNothingSent();
});

it('skips inactive jump-range alerts', function () {
    $staging = makeSolarsystem(30009113, -0.3, 'eve');
    $exit = makeSolarsystem(30009114, -0.1, 'eve', lightyears(2.0));

    $map = Map::factory()->create();
    jumpRangeAlert($map, ['target_solarsystem_id' => $staging, 'is_active' => false]);

    runJumpRangeEval($map, $exit);

    Http::assertNothingSent();
});

it('does not fire when the added system is the staging system itself', function () {
    $staging = makeSolarsystem(30009115, -0.3, 'eve');

    $map = Map::factory()->create();
    jumpRangeAlert($map, ['target_solarsystem_id' => $staging]);

    runJumpRangeEval($map, $staging);

    Http::assertNothingSent();
});

it('still evaluates proximity alerts in the same run', function () {
    $staging = makeSolarsystem(30009116, -0.3, 'eve');
    $exit = makeSolarsystem(30009117, -0.1, 'eve', lightyears(2.0));

    $map = Map::factory()->create();
    jumpRangeAlert($map, ['target_solarsystem_id' => $staging]);

    $webhook = MapWebhook::factory()->for($map)->create();
    MapAlert::factory()->create([
        'map_id' => $map->id,
        'map_webhook_id' => $webhook->id,
        'target_solarsystem_id' => $exit,
        'max_jumps' => 3,
    ]);

    runJumpRangeEval($map, $exit);

    Http::assertSentCount(2);
});
