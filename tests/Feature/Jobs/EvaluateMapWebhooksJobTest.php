<?php

declare(strict_types=1);

use App\Enums\MapAlertMentionMode;
use App\Jobs\MapAlerts\EvaluateMapAlertsJob;
use App\Models\Map;
use App\Models\MapAlert;
use App\Models\MapSolarsystem;
use App\Models\MapWebhook;
use App\Models\MapWebhookRole;
use Illuminate\Support\Facades\Http;

function runWebhookEval(Map $map, int $solarsystemId): void
{
    $placement = MapSolarsystem::query()->where('map_id', $map->id)->where('solarsystem_id', $solarsystemId)->first()
        ?? MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $solarsystemId]);
    app()->call([new EvaluateMapAlertsJob($placement->id), 'handle']);
}

/**
 * A proximity alert on the map, delivering to a fresh webhook destination.
 *
 * @param  array<string, mixed>  $attributes
 */
function proximityAlert(Map $map, array $attributes = []): MapAlert
{
    $webhook = MapWebhook::factory()->for($map)->create();

    return MapAlert::factory()->create(array_merge([
        'map_id' => $map->id,
        'map_webhook_id' => $webhook->id,
    ], $attributes));
}

beforeEach(function () {
    Http::fake();
});

it('fires when the added system is within range of the target', function () {
    $sid = makeSolarsystem(30009001);
    $map = Map::factory()->create();
    $alert = proximityAlert($map, ['target_solarsystem_id' => $sid, 'max_jumps' => 3]);

    runWebhookEval($map, $sid);

    Http::assertSentCount(1);

    expect($alert->refresh()->last_fired_at)->not->toBeNull();
});

it('does not fire when the added system is out of range', function () {
    $origin = makeSolarsystem(30009002);
    $target = makeSolarsystem(30009003);

    $map = Map::factory()->create();
    proximityAlert($map, ['target_solarsystem_id' => $target, 'max_jumps' => 3]);

    runWebhookEval($map, $origin);

    Http::assertNothingSent();
});

it('skips inactive alerts', function () {
    $sid = makeSolarsystem(30009001);
    $map = Map::factory()->create();
    proximityAlert($map, ['target_solarsystem_id' => $sid, 'max_jumps' => 3, 'is_active' => false]);

    runWebhookEval($map, $sid);

    Http::assertNothingSent();
});

it('ignores bot alerts without changing their last fired timestamp', function () {
    $sid = makeSolarsystem(30009012);
    $map = Map::factory()->create();
    $alert = MapAlert::factory()->discordDm()->create([
        'map_id' => $map->id,
        'target_solarsystem_id' => $sid,
        'max_jumps' => 3,
    ]);

    runWebhookEval($map, $sid);

    Http::assertNothingSent();
    expect($alert->refresh()->last_fired_at)->toBeNull();
});

it('fires across a real stargate hop', function () {
    /** @var array<int|string, int[]> $raw */
    $raw = json_decode(file_get_contents(resource_path('static/connections.json')), true);
    $origin = (int) array_key_first($raw);
    $target = (int) $raw[$origin][0];

    makeSolarsystem($origin);
    makeSolarsystem($target);

    $map = Map::factory()->create();
    proximityAlert($map, ['target_solarsystem_id' => $target, 'max_jumps' => 1]);

    runWebhookEval($map, $origin);

    Http::assertSentCount(1);
});

it('colours the embed green for a highsec target', function () {
    $sid = makeSolarsystem(30009010, 0.9);
    $map = Map::factory()->create();
    proximityAlert($map, ['target_solarsystem_id' => $sid, 'max_jumps' => 1]);

    runWebhookEval($map, $sid);

    Http::assertSent(fn ($request): bool => $request->data()['embeds'][0]['color'] === 0x2ECC71);
});

it('colours the embed red for a nullsec target', function () {
    $sid = makeSolarsystem(30009011, -0.5);
    $map = Map::factory()->create();
    proximityAlert($map, ['target_solarsystem_id' => $sid, 'max_jumps' => 1]);

    runWebhookEval($map, $sid);

    Http::assertSent(fn ($request): bool => $request->data()['embeds'][0]['color'] === 0xE74C3C);
});

it('does not count wormhole connections, only k-space jumps', function () {
    // A J-space system has no stargates, so even though it is the added system it can
    // never be within gate range of a k-space target.
    $origin = makeSolarsystem(31000001);
    $target = makeSolarsystem(30009005);

    $map = Map::factory()->create();
    proximityAlert($map, ['target_solarsystem_id' => $target, 'max_jumps' => 20]);

    runWebhookEval($map, $origin);

    Http::assertNothingSent();
});

it('pings a user mention through the webhook', function () {
    $sid = makeSolarsystem(30009021);
    $map = Map::factory()->create();
    $mention = MapWebhookRole::factory()->user()->for($map)->create(['discord_role_id' => '111222333']);
    proximityAlert($map, [
        'target_solarsystem_id' => $sid,
        'max_jumps' => 3,
        'map_webhook_role_id' => $mention->id,
    ]);

    runWebhookEval($map, $sid);

    Http::assertSent(fn ($request): bool => $request['content'] === '<@111222333>'
        && $request['allowed_mentions'] === ['users' => ['111222333']]);
});

it('pings a role mention through the webhook', function () {
    $sid = makeSolarsystem(30009022);
    $map = Map::factory()->create();
    $mention = MapWebhookRole::factory()->for($map)->create(['discord_role_id' => '444555666']);
    proximityAlert($map, [
        'target_solarsystem_id' => $sid,
        'max_jumps' => 3,
        'map_webhook_role_id' => $mention->id,
    ]);

    runWebhookEval($map, $sid);

    Http::assertSent(fn ($request): bool => $request['content'] === '<@&444555666>'
        && $request['allowed_mentions'] === ['roles' => ['444555666']]);
});

it('pings everyone when the alert mentions everyone', function () {
    $sid = makeSolarsystem(30009023);
    $map = Map::factory()->create();
    proximityAlert($map, [
        'target_solarsystem_id' => $sid,
        'max_jumps' => 3,
        'mention_mode' => MapAlertMentionMode::Everyone,
    ]);

    runWebhookEval($map, $sid);

    Http::assertSent(fn ($request): bool => $request['content'] === '@everyone'
        && $request['allowed_mentions'] === ['parse' => ['everyone']]);
});

it('does not deliver a webhook alert twice for the same placement', function () {
    $sid = makeSolarsystem(30009024);
    $map = Map::factory()->create();
    $alert = proximityAlert($map, ['target_solarsystem_id' => $sid, 'max_jumps' => 3]);

    runWebhookEval($map, $sid);
    runWebhookEval($map, $sid);

    Http::assertSentCount(1);
    expect($alert->deliveries()->count())->toBe(1);
});
