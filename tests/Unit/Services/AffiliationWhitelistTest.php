<?php

declare(strict_types=1);

use App\Services\AffiliationWhitelist;

function whitelist(array $allowed): AffiliationWhitelist
{
    return new AffiliationWhitelist($allowed);
}

it('is not enforced when no ids are configured', function () {
    expect(whitelist([])->isEnforced())->toBeFalse();
});

it('is enforced when ids are configured', function () {
    expect(whitelist([123])->isEnforced())->toBeTrue();
});

it('allows any affiliation when the whitelist is empty', function () {
    expect(whitelist([])->allows([1, 2, 3]))->toBeTrue();
});

it('allows a matching affiliation id', function (array $allowed, array $affiliations) {
    expect(whitelist($allowed)->allows($affiliations))->toBeTrue();
})->with([
    'character match' => [[42], [42, 2000, 3000]],
    'corporation match' => [[2000], [42, 2000, 3000]],
    'alliance match' => [[3000], [42, 2000, 3000]],
    'one of many allowed' => [[111, 2000, 999], [42, 2000, 3000]],
]);

it('denies when no affiliation matches the whitelist', function () {
    expect(whitelist([111, 222])->allows([42, 2000, 3000]))->toBeFalse();
});

it('ignores null affiliations when matching', function () {
    expect(whitelist([3000])->allows([42, 2000, null]))->toBeFalse();
    expect(whitelist([42])->allows([42, 2000, null]))->toBeTrue();
});
