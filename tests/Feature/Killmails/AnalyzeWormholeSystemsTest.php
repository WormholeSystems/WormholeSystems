<?php

declare(strict_types=1);

use App\Collections\OrganisationStatsCollection;
use App\DTO\AnalysisOptions;
use App\DTO\SystemAnalysisResult;
use App\Enums\ThreatLevel;

function makeAnalysisOptions(
    int $hostileThreshold = 50,
    int $activeThreshold = 15,
): AnalysisOptions {
    return new AnalysisOptions(
        daysAgo: 90,
        daysActive: 1,
        top: 10,
        activeThreshold: $activeThreshold,
        hostileThreshold: $hostileThreshold,
    );
}

function makeOrganisationStats(int $killCount): OrganisationStatsCollection
{
    $stats = new OrganisationStatsCollection();

    for ($i = 0; $i < $killCount; $i++) {
        $stats->addActivity(1000001, 'corporation', now()->subDays($i)->format('Y-m-d'));
    }

    return $stats;
}

it('assigns critical threat level when kills exceed hostile threshold', function () {
    $result = SystemAnalysisResult::create(
        makeOrganisationStats(60),
        makeAnalysisOptions(),
    );

    expect($result->threatLevel)->toBe(ThreatLevel::Critical)
        ->and($result->totalKills)->toBe(60);
});

it('assigns high threat level when kills exceed active threshold', function () {
    $result = SystemAnalysisResult::create(
        makeOrganisationStats(20),
        makeAnalysisOptions(),
    );

    expect($result->threatLevel)->toBe(ThreatLevel::High);
});

it('assigns unknown threat level when kills are below active threshold', function () {
    $result = SystemAnalysisResult::create(
        makeOrganisationStats(8),
        makeAnalysisOptions(),
    );

    expect($result->threatLevel)->toBe(ThreatLevel::Unknown);
});

it('assigns unknown threat level when no kills', function () {
    $result = SystemAnalysisResult::create(
        new OrganisationStatsCollection(),
        makeAnalysisOptions(),
    );

    expect($result->threatLevel)->toBe(ThreatLevel::Unknown)
        ->and($result->totalKills)->toBe(0)
        ->and($result->threatData)->toBe([]);
});

it('generates structured threat data with entity details', function () {
    $result = SystemAnalysisResult::create(
        makeOrganisationStats(20),
        makeAnalysisOptions(),
    );

    expect($result->threatData)->toBeArray()
        ->and($result->threatData)->toHaveCount(1)
        ->and($result->threatData[0])->toHaveKeys(['id', 'name', 'type', 'kills']);
});

it('persists the analyzed threat entities and replaces them on re-analysis', function () {
    makeSolarsystem(31000501);
    $wormholeSystem = App\Models\WormholeSystem::query()->create(['id' => 31000501]);
    App\Models\Corporation::query()->create(['id' => 98000401, 'name' => 'Threat Corp']);

    foreach (range(1, 3) as $index) {
        App\Models\Killmail::query()->create([
            'id' => 910000 + $index,
            'hash' => 'hash-analyze-'.$index,
            'time' => now()->subDays($index),
            'solarsystem_id' => 31000501,
            'data' => [
                'victim' => ['corporation_id' => 98000401],
                'attackers' => [],
            ],
            'zkb' => [],
        ]);
    }

    $this->artisan('app:analyze-wormhole-systems', ['--days-active' => 1])->assertSuccessful();

    $threats = $wormholeSystem->refresh()->threats;

    expect($threats)->toHaveCount(1)
        ->and($threats->first()->entity_id)->toBe(98000401)
        ->and($threats->first()->entity_type)->toBe('corporation')
        ->and($threats->first()->name)->toBe('Threat Corp')
        ->and($threats->first()->kills)->toBe(3);

    $this->artisan('app:analyze-wormhole-systems', ['--days-active' => 1])->assertSuccessful();

    expect($wormholeSystem->refresh()->threats)->toHaveCount(1);
});
