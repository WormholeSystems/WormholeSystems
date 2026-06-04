<?php

declare(strict_types=1);

use App\Enums\KillmailFilterMatch;
use App\Enums\KillmailFilterMode;
use App\Enums\KillmailFilterSide;
use App\Enums\KillmailFilterSubject;
use App\Services\Killmails\KillmailFilterRule;
use App\Services\Killmails\KillmailWebhookMatcher;
use Illuminate\Support\Collection;

function killmailData(): array
{
    return [
        'victim' => ['character_id' => 100, 'corporation_id' => 200, 'alliance_id' => 300, 'ship_type_id' => 587],
        'attackers' => [
            ['character_id' => 101, 'corporation_id' => 201, 'alliance_id' => 301, 'ship_type_id' => 11567, 'final_blow' => true],
            ['corporation_id' => 202, 'ship_type_id' => 670],
        ],
    ];
}

function rule(KillmailFilterSubject $subject, KillmailFilterSide $side, KillmailFilterMode $mode, array $ids): KillmailFilterRule
{
    return new KillmailFilterRule($subject, $side, $mode, $ids);
}

/**
 * @param  KillmailFilterRule[]  $rules
 */
function matchKillmail(array $rules, array $typeGroupMap = [], ?array $data = null, KillmailFilterMatch $match = KillmailFilterMatch::Any): bool
{
    $matcher = new KillmailWebhookMatcher;
    $pools = $matcher->buildPools($data ?? killmailData(), $typeGroupMap);

    return $matcher->matches($pools, new Collection($rules), $match);
}

/**
 * @param  KillmailFilterRule[]  $rules
 * @return Collection<int, KillmailFilterRule>
 */
function matchingRulesFor(array $rules, array $typeGroupMap = []): Collection
{
    $matcher = new KillmailWebhookMatcher;

    return $matcher->matchingRules($matcher->buildPools(killmailData(), $typeGroupMap), new Collection($rules));
}

it('matches everything when there are no filters', function () {
    expect(matchKillmail([]))->toBeTrue();
});

it('matches an include rule against the victim', function () {
    expect(matchKillmail([rule(KillmailFilterSubject::Character, KillmailFilterSide::Victim, KillmailFilterMode::Include, [100])]))->toBeTrue();
});

it('fails an include rule when the victim id is absent', function () {
    expect(matchKillmail([rule(KillmailFilterSubject::Character, KillmailFilterSide::Victim, KillmailFilterMode::Include, [999])]))->toBeFalse();
});

it('matches an include rule against an attacker', function () {
    expect(matchKillmail([rule(KillmailFilterSubject::Character, KillmailFilterSide::Attacker, KillmailFilterMode::Include, [101])]))->toBeTrue();
});

it('does not match an attacker id when restricted to the victim side', function () {
    expect(matchKillmail([rule(KillmailFilterSubject::Character, KillmailFilterSide::Victim, KillmailFilterMode::Include, [101])]))->toBeFalse();
});

it('matches either side', function () {
    expect(matchKillmail([rule(KillmailFilterSubject::Corporation, KillmailFilterSide::Either, KillmailFilterMode::Include, [202])]))->toBeTrue();
});

it('vetoes when an exclude rule matches', function () {
    expect(matchKillmail([rule(KillmailFilterSubject::Alliance, KillmailFilterSide::Either, KillmailFilterMode::Exclude, [301])]))->toBeFalse();
});

it('passes an exclude rule when its id is absent', function () {
    expect(matchKillmail([rule(KillmailFilterSubject::Alliance, KillmailFilterSide::Either, KillmailFilterMode::Exclude, [999])]))->toBeTrue();
});

it('matches ship type', function () {
    expect(matchKillmail([rule(KillmailFilterSubject::ShipType, KillmailFilterSide::Attacker, KillmailFilterMode::Include, [11567])]))->toBeTrue();
});

it('matches ship group using the resolved type-group map', function () {
    $map = [587 => 25, 11567 => 26, 670 => 27];

    expect(matchKillmail([rule(KillmailFilterSubject::ShipGroup, KillmailFilterSide::Victim, KillmailFilterMode::Include, [25])], $map))->toBeTrue()
        ->and(matchKillmail([rule(KillmailFilterSubject::ShipGroup, KillmailFilterSide::Attacker, KillmailFilterMode::Include, [26])], $map))->toBeTrue()
        ->and(matchKillmail([rule(KillmailFilterSubject::ShipGroup, KillmailFilterSide::Victim, KillmailFilterMode::Include, [99])], $map))->toBeFalse();
});

it('ORs multiple ids within a single rule', function () {
    expect(matchKillmail([rule(KillmailFilterSubject::Corporation, KillmailFilterSide::Either, KillmailFilterMode::Include, [999, 202])]))->toBeTrue();
});

it('ORs multiple include rules in any mode', function () {
    // Victim is a battleship (group 25), not a pod (group 29) — "any" should still match.
    $rules = [
        rule(KillmailFilterSubject::ShipGroup, KillmailFilterSide::Victim, KillmailFilterMode::Include, [25]),
        rule(KillmailFilterSubject::ShipGroup, KillmailFilterSide::Victim, KillmailFilterMode::Include, [29]),
    ];

    expect(matchKillmail($rules, [587 => 25], match: KillmailFilterMatch::Any))->toBeTrue();
});

it('ANDs multiple include rules in all mode', function () {
    $rules = [
        rule(KillmailFilterSubject::Character, KillmailFilterSide::Victim, KillmailFilterMode::Include, [100]),
        rule(KillmailFilterSubject::Corporation, KillmailFilterSide::Attacker, KillmailFilterMode::Include, [201]),
    ];

    expect(matchKillmail($rules, match: KillmailFilterMatch::All))->toBeTrue();

    $rules[1] = rule(KillmailFilterSubject::Corporation, KillmailFilterSide::Attacker, KillmailFilterMode::Include, [999]);

    expect(matchKillmail($rules, match: KillmailFilterMatch::All))->toBeFalse()
        // The same rules in "any" mode still match because the first rule matches.
        ->and(matchKillmail($rules, match: KillmailFilterMatch::Any))->toBeTrue();
});

it('reports only the include rules that matched', function () {
    $matching = rule(KillmailFilterSubject::Corporation, KillmailFilterSide::Victim, KillmailFilterMode::Include, [200]);
    $notMatching = rule(KillmailFilterSubject::Corporation, KillmailFilterSide::Victim, KillmailFilterMode::Include, [999]);
    $exclude = rule(KillmailFilterSubject::Alliance, KillmailFilterSide::Victim, KillmailFilterMode::Exclude, [300]);

    $matched = matchingRulesFor([$matching, $notMatching, $exclude]);

    expect($matched)->toHaveCount(1)
        ->and($matched->first()->subject)->toBe(KillmailFilterSubject::Corporation)
        ->and($matched->first()->ids)->toBe([200]);
});

it('narrows a matched rule to only the ids that matched', function () {
    // Victim ship type 587 resolves to group 25; the rule lists groups 25 and 99.
    $rule = rule(KillmailFilterSubject::ShipGroup, KillmailFilterSide::Victim, KillmailFilterMode::Include, [25, 99]);

    $matched = matchingRulesFor([$rule], [587 => 25]);

    expect($matched)->toHaveCount(1)
        ->and($matched->first()->ids)->toBe([25]);
});

it('always vetoes on an exclude rule regardless of match mode', function () {
    $rules = [
        rule(KillmailFilterSubject::Character, KillmailFilterSide::Victim, KillmailFilterMode::Include, [100]),
        rule(KillmailFilterSubject::Alliance, KillmailFilterSide::Attacker, KillmailFilterMode::Exclude, [301]),
    ];

    expect(matchKillmail($rules, match: KillmailFilterMatch::Any))->toBeFalse()
        ->and(matchKillmail($rules, match: KillmailFilterMatch::All))->toBeFalse();
});

it('handles attackers that are missing fields without matching falsely', function () {
    // The second attacker has no character_id; an include on character 102 must not match.
    expect(matchKillmail([rule(KillmailFilterSubject::Character, KillmailFilterSide::Attacker, KillmailFilterMode::Include, [102])]))->toBeFalse()
        // ...but the corporation it does carry still matches.
        ->and(matchKillmail([rule(KillmailFilterSubject::Corporation, KillmailFilterSide::Attacker, KillmailFilterMode::Include, [202])]))->toBeTrue();
});
