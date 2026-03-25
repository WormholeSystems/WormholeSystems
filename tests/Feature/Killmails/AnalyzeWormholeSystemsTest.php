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
