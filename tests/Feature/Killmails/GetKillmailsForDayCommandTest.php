<?php

declare(strict_types=1);

use App\Models\Alliance;
use App\Models\Corporation;
use App\Models\Killmail;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Storage;

/**
 * @param  array<int, array<string, mixed>>  $killmails
 */
function fakeKillmailArchive(array $killmails): string
{
    $directory = sys_get_temp_dir().'/killmail-archive-'.uniqid();
    mkdir($directory);

    $archive = new PharData($directory.'/killmails.tar');

    foreach ($killmails as $killmail) {
        $archive->addFromString(sprintf('killmails/%d.json', $killmail['killmail_id']), (string) json_encode($killmail));
    }

    $archive->compress(Phar::BZ2);

    return $directory.'/killmails.tar.bz2';
}

/**
 * @return array<string, mixed>
 */
function archiveKillmail(int $id, int $solarsystemId, ?int $corporationId = null, ?int $allianceId = null): array
{
    return [
        'killmail_id' => $id,
        'killmail_hash' => 'hash-'.$id,
        'killmail_time' => '2026-04-08T12:00:00Z',
        'solar_system_id' => $solarsystemId,
        'victim' => array_filter([
            'corporation_id' => $corporationId,
            'alliance_id' => $allianceId,
        ]),
        'attackers' => [],
    ];
}

it('imports killmails from the daily archive and stores bare victim organisations', function () {
    Storage::fake();
    makeSolarsystem(31000401);

    $archive = fakeKillmailArchive([
        archiveKillmail(900001, 31000401, corporationId: 98000301, allianceId: 99000301),
    ]);

    Http::fake(['data.everef.net/*' => Http::response(file_get_contents($archive))]);

    $this->artisan('app:get-killmails-for-day', ['date' => '2026-04-08'])->assertSuccessful();

    expect(Killmail::query()->find(900001))
        ->not->toBeNull()
        ->solarsystem_id->toBe(31000401);

    expect(Corporation::query()->find(98000301))->not->toBeNull()->name->toBeNull();
    expect(Alliance::query()->find(99000301))->not->toBeNull()->name->toBeNull();
});

it('never overwrites zkb metadata stored by the live listener', function () {
    Storage::fake();
    makeSolarsystem(31000402);

    Killmail::query()->create([
        'id' => 900002,
        'hash' => 'hash-900002',
        'time' => '2026-04-09 12:00:00',
        'solarsystem_id' => 31000402,
        'data' => ['killmail_id' => 900002],
        'zkb' => ['totalValue' => 1234567.89],
    ]);

    $archive = fakeKillmailArchive([
        archiveKillmail(900002, 31000402),
    ]);

    Http::fake(['data.everef.net/*' => Http::response(file_get_contents($archive))]);

    $this->artisan('app:get-killmails-for-day', ['date' => '2026-04-09'])->assertSuccessful();

    expect(Killmail::query()->find(900002)?->zkb)->toMatchArray(['totalValue' => 1234567.89]);
});

it('reports success without storing anything when the archive does not exist', function () {
    Storage::fake();

    Http::fake(['data.everef.net/*' => Http::response(null, 404)]);

    $this->artisan('app:get-killmails-for-day', ['date' => '2026-04-10'])->assertSuccessful();

    expect(Killmail::query()->count())->toBe(0);
});
