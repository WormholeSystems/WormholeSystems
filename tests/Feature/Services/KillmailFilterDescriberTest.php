<?php

declare(strict_types=1);

use App\Enums\KillmailFilterMode;
use App\Enums\KillmailFilterSide;
use App\Enums\KillmailFilterSubject;
use App\Models\Corporation;
use App\Services\Killmails\KillmailFilterDescriber;
use App\Services\Killmails\KillmailFilterRule;
use Illuminate\Support\Collection;

it('resolves entity ids to names and labels the rule', function () {
    $corp = Corporation::factory()->create(['name' => 'Goonswarm Federation']);

    $lines = (new KillmailFilterDescriber)->describe(new Collection([
        new KillmailFilterRule(KillmailFilterSubject::Corporation, KillmailFilterSide::Attacker, KillmailFilterMode::Include, [$corp->id]),
    ]));

    expect($lines)->toBe(['Corporation (attacker) — Must match: Goonswarm Federation']);
});

it('falls back to the raw id when no record matches', function () {
    $lines = (new KillmailFilterDescriber)->describe(new Collection([
        new KillmailFilterRule(KillmailFilterSubject::ShipGroup, KillmailFilterSide::Either, KillmailFilterMode::Exclude, [987654]),
    ]));

    expect($lines)->toBe(['Group (either side) — Must not match: 987654']);
});

it('returns no lines for an empty filter set', function () {
    expect((new KillmailFilterDescriber)->describe(new Collection))->toBe([]);
});
