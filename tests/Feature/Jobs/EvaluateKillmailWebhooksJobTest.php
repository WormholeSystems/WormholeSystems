<?php

declare(strict_types=1);

use App\Jobs\Webhooks\EvaluateKillmailWebhooksJob;
use App\Models\Killmail;
use App\Models\Map;
use App\Models\MapSolarsystem;
use App\Models\MapWebhook;
use Illuminate\Support\Facades\Http;

function makeKillmail(int $solarsystemId, array $overrides = []): Killmail
{
    $id = fake()->unique()->numberBetween(1, 2_000_000_000);

    return Killmail::query()->create([
        'id' => $id,
        'hash' => fake()->sha1(),
        'solarsystem_id' => $solarsystemId,
        'time' => now(),
        'data' => array_merge([
            'killmail_id' => $id,
            'solar_system_id' => $solarsystemId,
            'victim' => ['character_id' => 100, 'corporation_id' => 200, 'alliance_id' => 300, 'ship_type_id' => 587],
            'attackers' => [
                ['character_id' => 101, 'corporation_id' => 201, 'alliance_id' => 301, 'ship_type_id' => 11567, 'final_blow' => true],
            ],
        ], $overrides),
        'zkb' => ['totalValue' => 1_500_000.0],
    ]);
}

function mapWithSystem(int $solarsystemId): Map
{
    $map = Map::factory()->create();
    MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $solarsystemId]);

    return $map;
}

function runKillmailEval(int $killmailId): void
{
    app()->call([new EvaluateKillmailWebhooksJob($killmailId), 'handle']);
}

beforeEach(function () {
    Http::fake();
});

it('fires when a matching kill occurs in a system on the map', function () {
    $sid = makeSolarsystem(30009401);
    $map = mapWithSystem($sid);
    $webhook = MapWebhook::factory()->for($map)->killmail([
        ['subject' => 'corporation', 'side' => 'either', 'mode' => 'include', 'ids' => [201]],
    ])->create(['max_jumps' => 3]);

    $killmail = makeKillmail($sid);

    runKillmailEval($killmail->id);

    Http::assertSentCount(1);
    expect($webhook->refresh()->last_fired_at)->not->toBeNull();
});

it('fires across a real stargate hop within range', function () {
    /** @var array<int|string, int[]> $raw */
    $raw = json_decode(file_get_contents(resource_path('static/connections.json')), true);
    $origin = (int) array_key_first($raw);
    $killSystem = (int) $raw[$origin][0];

    makeSolarsystem($origin);
    makeSolarsystem($killSystem);

    $map = Map::factory()->create();
    MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $origin, 'alias' => 'HOME']);
    MapWebhook::factory()->for($map)->killmail([
        ['subject' => 'corporation', 'side' => 'either', 'mode' => 'include', 'ids' => [201]],
    ])->create(['max_jumps' => 1]);

    $killmail = makeKillmail($killSystem);

    runKillmailEval($killmail->id);

    Http::assertSentCount(1);
    Http::assertSent(function ($request) {
        $embed = $request->data()['embeds'][0];
        $fieldNames = array_column($embed['fields'], 'name');

        $whenField = collect($embed['fields'])->firstWhere('name', 'When');

        return str_contains($embed['thumbnail']['url'] ?? '', '/types/')
            && in_array('Exit from', $fieldNames, true)
            && in_array('Route', $fieldNames, true)
            && in_array('Matched filters', $fieldNames, true)
            && ! empty($embed['timestamp'])
            && $whenField !== null
            && str_contains($whenField['value'], ':R>')
            && $embed['description'] !== null
            && ! str_contains($embed['description'], 'inside your chain');
    });
});

it('pings the configured role when one is set', function () {
    $sid = makeSolarsystem(30009610);
    $map = mapWithSystem($sid);
    MapWebhook::factory()->for($map)->killmail()->create(['max_jumps' => 3, 'discord_role_id' => '123456789']);

    runKillmailEval(makeKillmail($sid)->id);

    Http::assertSent(function ($request) {
        return ($request->data()['content'] ?? '') === '<@&123456789>'
            && ($request->data()['allowed_mentions']['roles'] ?? []) === ['123456789'];
    });
});

it('does not add a mention when no role is set', function () {
    $sid = makeSolarsystem(30009611);
    $map = mapWithSystem($sid);
    MapWebhook::factory()->for($map)->killmail()->create(['max_jumps' => 3]);

    runKillmailEval(makeKillmail($sid)->id);

    Http::assertSent(fn ($request) => ! array_key_exists('content', $request->data()));
});

it('marks an in-chain kill as inside the chain rather than a route', function () {
    $sid = makeSolarsystem(30009601);
    $map = mapWithSystem($sid);
    MapWebhook::factory()->for($map)->killmail()->create(['max_jumps' => 3]);

    $killmail = makeKillmail($sid);

    runKillmailEval($killmail->id);

    Http::assertSent(function ($request) {
        $embed = $request->data()['embeds'][0];

        return str_contains($embed['description'], 'inside your chain')
            && ! in_array('Route', array_column($embed['fields'], 'name'), true);
    });
});

it('does not fire for a kill beyond max jumps over real stargates', function () {
    /** @var array<int|string, int[]> $raw */
    $raw = json_decode(file_get_contents(resource_path('static/connections.json')), true);
    $origin = (int) array_key_first($raw);
    $oneHop = (int) $raw[$origin][0];

    // A system two hops away: a neighbour of $oneHop that is neither the origin nor a direct neighbour of it.
    $directNeighbours = array_map('intval', $raw[$origin]);
    $twoHop = null;
    foreach ($raw[$oneHop] as $candidate) {
        $candidate = (int) $candidate;
        if ($candidate !== $origin && ! in_array($candidate, $directNeighbours, true)) {
            $twoHop = $candidate;
            break;
        }
    }
    expect($twoHop)->not->toBeNull();

    makeSolarsystem($origin);
    makeSolarsystem($twoHop);

    $map = mapWithSystem($origin);
    MapWebhook::factory()->for($map)->killmail()->create(['max_jumps' => 1]);

    $killmail = makeKillmail($twoHop);

    runKillmailEval($killmail->id);

    Http::assertNothingSent();
});

it('does not treat an unplaced map solarsystem as being on the map', function () {
    $placed = makeSolarsystem(30009501);
    $unplaced = makeSolarsystem(30009502);

    $map = mapWithSystem($placed);
    // A tracked-but-unplaced system (no position) is not actually on the chain.
    MapSolarsystem::factory()->for($map)->create([
        'solarsystem_id' => $unplaced,
        'position_x' => null,
        'position_y' => null,
    ]);
    MapWebhook::factory()->for($map)->killmail()->create(['max_jumps' => 1]);

    $killmail = makeKillmail($unplaced);

    runKillmailEval($killmail->id);

    Http::assertNothingSent();
});

it('does not fire when the kill is out of range', function () {
    $mapSystem = makeSolarsystem(30009402);
    $killSystem = makeSolarsystem(30009403);
    $map = mapWithSystem($mapSystem);
    MapWebhook::factory()->for($map)->killmail()->create(['max_jumps' => 3]);

    $killmail = makeKillmail($killSystem);

    runKillmailEval($killmail->id);

    Http::assertNothingSent();
});

it('does not fire when filters do not match even if in range', function () {
    $sid = makeSolarsystem(30009404);
    $map = mapWithSystem($sid);
    MapWebhook::factory()->for($map)->killmail([
        ['subject' => 'corporation', 'side' => 'either', 'mode' => 'include', 'ids' => [999999]],
    ])->create(['max_jumps' => 3]);

    $killmail = makeKillmail($sid);

    runKillmailEval($killmail->id);

    Http::assertNothingSent();
});

it('fires in any-match mode when only one of several include filters matches', function () {
    $sid = makeSolarsystem(30009420);
    $map = mapWithSystem($sid);
    // Victim corp is 200; the second rule (999) never matches, but "any" only needs one.
    MapWebhook::factory()->for($map)->killmail([
        ['subject' => 'corporation', 'side' => 'victim', 'mode' => 'include', 'ids' => [200]],
        ['subject' => 'corporation', 'side' => 'victim', 'mode' => 'include', 'ids' => [999]],
    ])->create(['max_jumps' => 3, 'filter_match' => 'any']);

    runKillmailEval(makeKillmail($sid)->id);

    Http::assertSentCount(1);
});

it('does not fire in all-match mode unless every include filter matches', function () {
    $sid = makeSolarsystem(30009421);
    $map = mapWithSystem($sid);
    MapWebhook::factory()->for($map)->killmail([
        ['subject' => 'corporation', 'side' => 'victim', 'mode' => 'include', 'ids' => [200]],
        ['subject' => 'corporation', 'side' => 'victim', 'mode' => 'include', 'ids' => [999]],
    ])->create(['max_jumps' => 3, 'filter_match' => 'all']);

    runKillmailEval(makeKillmail($sid)->id);

    Http::assertNothingSent();
});

it('lists only the filters that actually matched the kill in the embed', function () {
    $sid = makeSolarsystem(30009422);
    $map = mapWithSystem($sid);
    // Victim corp is 200; the 999 rule does not match and must not appear in the embed.
    MapWebhook::factory()->for($map)->killmail([
        ['subject' => 'corporation', 'side' => 'victim', 'mode' => 'include', 'ids' => [200]],
        ['subject' => 'corporation', 'side' => 'victim', 'mode' => 'include', 'ids' => [999]],
    ])->create(['max_jumps' => 3, 'filter_match' => 'any']);

    runKillmailEval(makeKillmail($sid)->id);

    Http::assertSent(function ($request) {
        $matched = collect($request->data()['embeds'][0]['fields'])->firstWhere('name', 'Matched filters');

        return $matched !== null
            && str_contains($matched['value'], '200')
            && ! str_contains($matched['value'], '999');
    });
});

it('honours an exclude filter', function () {
    $sid = makeSolarsystem(30009405);
    $map = mapWithSystem($sid);
    MapWebhook::factory()->for($map)->killmail([
        ['subject' => 'alliance', 'side' => 'attacker', 'mode' => 'exclude', 'ids' => [301]],
    ])->create(['max_jumps' => 3]);

    $killmail = makeKillmail($sid);

    runKillmailEval($killmail->id);

    Http::assertNothingSent();
});

it('skips inactive killmail webhooks', function () {
    $sid = makeSolarsystem(30009406);
    $map = mapWithSystem($sid);
    MapWebhook::factory()->for($map)->killmail()->inactive()->create(['max_jumps' => 3]);

    $killmail = makeKillmail($sid);

    runKillmailEval($killmail->id);

    Http::assertNothingSent();
});

it('ignores proximity webhooks and returns early when no killmail webhooks exist', function () {
    $sid = makeSolarsystem(30009407);
    $map = mapWithSystem($sid);
    MapWebhook::factory()->for($map)->create(['target_solarsystem_id' => $sid, 'max_jumps' => 3]);

    $killmail = makeKillmail($sid);

    runKillmailEval($killmail->id);

    Http::assertNothingSent();
});

it('builds a stable unique id for de-duplication', function () {
    expect((new EvaluateKillmailWebhooksJob(42))->uniqueId())->toBe('killmail:42');
});
