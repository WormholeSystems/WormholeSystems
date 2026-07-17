<?php

declare(strict_types=1);

use App\Models\Killmail;
use App\Models\User;

function makeLandingKillmail(int $id, int $solarsystemId): Killmail
{
    return Killmail::query()->forceCreate([
        'id' => $id,
        'hash' => 'hash-'.$id,
        'solarsystem_id' => $solarsystemId,
        'time' => now(),
        'data' => [
            'victim' => [
                'character_id' => 90000001,
                'corporation_id' => null,
                'alliance_id' => null,
                'faction_id' => null,
                'ship_type_id' => 670,
                'items' => [],
            ],
            'attackers' => [],
        ],
        'zkb' => ['totalValue' => 10_000_000, 'points' => 1],
    ]);
}

it('renders the landing page for guests', function () {
    $response = $this->get(route('landing'));

    $response->assertSuccessful();
    $response->assertInertia(fn ($page) => $page
        ->component('Landing')
        ->has('killmails'));
});

it('serves the latest wormhole killmails', function () {
    $wormholeSystem = makeSolarsystem(31000001, -1.0, 'wormhole');
    $knownSpaceSystem = makeSolarsystem(30000142, 0.9, 'eve');

    $wormholeKillmail = makeLandingKillmail(1001, $wormholeSystem);
    makeLandingKillmail(1002, $knownSpaceSystem);

    $response = $this->get(route('landing'));

    $response->assertInertia(fn ($page) => $page
        ->component('Landing')
        ->has('killmails', 1)
        ->where('killmails.0.id', $wormholeKillmail->id));
});

it('redirects authenticated users away from the landing page', function () {
    $this->actingAs(User::factory()->create());

    $response = $this->get(route('landing'));

    $response->assertRedirect();
});
