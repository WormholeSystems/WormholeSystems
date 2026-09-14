<?php

declare(strict_types=1);

use App\Actions\Discord\CreateDiscordAlertAction;
use App\Actions\Discord\DiscordAlertFormatter;
use App\Actions\Statistics\FinalizeMaintainerReportAction;
use App\Enums\MapAlertDeliveryType;
use App\Enums\MapAlertMentionMode;
use App\Enums\MapAlertType;
use App\Enums\Permission;
use App\Enums\SignatureActivityAction;
use App\Models\Character;
use App\Models\DiscordAccount;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapAlert;
use App\Models\MapMaintainerReport;
use App\Models\MapWebhook;
use App\Models\SignatureActivity;
use App\Models\User;
use App\Services\Statistics\MaintainerPeriod;
use Carbon\CarbonImmutable;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Log;

use function Pest\Laravel\actingAs;
use function Pest\Laravel\travelTo;

function podiumMap(array $attributes = []): Map
{
    return Map::factory()->create(array_merge([
        'maintainer_points_created' => 1,
        'maintainer_points_updated' => 1,
        'maintainer_points_deleted' => 1,
        'maintainer_minimum_points' => 0,
    ], $attributes))->fresh();
}

function podiumAlert(Map $map, array $attributes = []): MapAlert
{
    $webhook = MapWebhook::factory()->for($map)->create();

    return MapAlert::factory()->maintainerPodium()->create(array_merge([
        'map_id' => $map->id,
        'map_webhook_id' => $webhook->id,
    ], $attributes));
}

function recordCreated(Map $map, Character $character, string $date): void
{
    SignatureActivity::query()->create([
        'map_id' => $map->id,
        'character_id' => $character->id,
        'signature_id' => random_int(1, 1_000_000_000),
        'solarsystem_id' => 30000001,
        'dedupe_key' => 'pk:'.$character->id,
        'action' => SignatureActivityAction::Created,
        'activity_date' => $date,
        'created_at' => CarbonImmutable::parse($date, 'UTC'),
    ]);
}

function maintainerAlertManager(Map $map): User
{
    return User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Manager])->for($map)))
        ->create();
}

beforeEach(function () {
    travelTo(CarbonImmutable::parse('2026-07-15 00:00:00', 'UTC'));
});

it('finalizes every map but only posts to Discord for maps with an active podium alert', function () {
    $withAlert = podiumMap();
    podiumAlert($withAlert);
    recordCreated($withAlert, Character::factory()->create(), '2026-06-10');

    $withoutAlert = podiumMap();

    Http::fake();
    $this->artisan('app:post-maintainer-podium')->assertSuccessful();

    Http::assertSentCount(1);
    expect(MapMaintainerReport::query()->where('map_id', $withAlert->id)->where('period', '2026-06')->exists())->toBeTrue()
        ->and(MapMaintainerReport::query()->where('map_id', $withoutAlert->id)->where('period', '2026-06')->exists())->toBeTrue()
        ->and(MapMaintainerReport::query()->where('map_id', $withoutAlert->id)->first()->discord_posted_at)->toBeNull();
});

it('does not post for an inactive podium alert', function () {
    $map = podiumMap();
    podiumAlert($map, ['is_active' => false]);
    recordCreated($map, Character::factory()->create(), '2026-06-10');

    Http::fake();
    $this->artisan('app:post-maintainer-podium')->assertSuccessful();

    Http::assertNothingSent();
});

it('caps the podium at 10 lines and adds a full-list link when more than 10 qualify', function () {
    $map = podiumMap();
    podiumAlert($map);

    foreach (range(1, 12) as $i) {
        recordCreated($map, Character::factory()->create(), '2026-06-10');
    }

    Http::fake();
    $this->artisan('app:post-maintainer-podium')->assertSuccessful();

    Http::assertSentCount(1);
    Http::assertSent(function ($request): bool {
        $embed = $request->data()['embeds'][0];

        return mb_substr_count($embed['description'], ' pts (') === 10
            && ($embed['fields'][0]['name'] ?? null) === 'Full list';
    });
});

it('shows medals only for the positions that exist when fewer than three qualify', function () {
    $map = podiumMap();
    podiumAlert($map);
    recordCreated($map, Character::factory()->create(), '2026-06-10');
    recordCreated($map, Character::factory()->create(), '2026-06-11');

    Http::fake();
    $this->artisan('app:post-maintainer-podium')->assertSuccessful();

    Http::assertSent(function ($request): bool {
        $description = $request->data()['embeds'][0]['description'];

        return str_contains($description, '🥇')
            && str_contains($description, '🥈')
            && ! str_contains($description, '🥉');
    });
});

it('still posts a valid embed when nobody qualifies', function () {
    $map = podiumMap(['maintainer_minimum_points' => 100]);
    podiumAlert($map);
    recordCreated($map, Character::factory()->create(), '2026-06-10');

    Http::fake();
    $this->artisan('app:post-maintainer-podium')->assertSuccessful();

    Http::assertSentCount(1);
    Http::assertSent(fn ($request): bool => $request->data()['embeds'][0]['description'] === 'Nobody reached the minimum points for this month\'s recap.');
});

it('excludes a sub-threshold scorer from the Discord podium', function () {
    $map = podiumMap(['maintainer_minimum_points' => 2]);
    podiumAlert($map);

    $below = Character::factory()->create(['name' => 'Below Threshold']);
    $above = Character::factory()->create(['name' => 'Above Threshold']);
    recordCreated($map, $below, '2026-06-10');
    SignatureActivity::query()->create([
        'map_id' => $map->id,
        'character_id' => $above->id,
        'signature_id' => random_int(1, 1_000_000_000),
        'solarsystem_id' => 30000001,
        'dedupe_key' => 'pk:'.$above->id.':1',
        'action' => SignatureActivityAction::Created,
        'activity_date' => '2026-06-10',
        'created_at' => CarbonImmutable::parse('2026-06-10', 'UTC'),
    ]);
    SignatureActivity::query()->create([
        'map_id' => $map->id,
        'character_id' => $above->id,
        'signature_id' => random_int(1, 1_000_000_000),
        'solarsystem_id' => 30000001,
        'dedupe_key' => 'pk:'.$above->id.':2',
        'action' => SignatureActivityAction::Created,
        'activity_date' => '2026-06-11',
        'created_at' => CarbonImmutable::parse('2026-06-11', 'UTC'),
    ]);

    Http::fake();
    $this->artisan('app:post-maintainer-podium')->assertSuccessful();

    Http::assertSent(function ($request) use ($below, $above): bool {
        $description = $request->data()['embeds'][0]['description'];

        return str_contains($description, $above->name)
            && ! str_contains($description, $below->name);
    });
});

it('adds a full-list link when a sub-threshold scorer is excluded, even under the 10-line cap', function () {
    $map = podiumMap(['maintainer_minimum_points' => 2]);
    podiumAlert($map);

    recordCreated($map, Character::factory()->create(), '2026-06-10');

    Http::fake();
    $this->artisan('app:post-maintainer-podium')->assertSuccessful();

    Http::assertSent(function ($request): bool {
        $embed = $request->data()['embeds'][0];

        return $embed['description'] === 'Nobody reached the minimum points for this month\'s recap.'
            && ($embed['fields'][0]['name'] ?? null) === 'Full list';
    });
});

it('does not post twice when run twice for the same period', function () {
    $map = podiumMap();
    podiumAlert($map);
    recordCreated($map, Character::factory()->create(), '2026-06-10');

    Http::fake();
    $this->artisan('app:post-maintainer-podium')->assertSuccessful();
    $this->artisan('app:post-maintainer-podium')->assertSuccessful();

    Http::assertSentCount(1);
});

it('does not re-post when re-run with an explicit already-posted period', function () {
    $map = podiumMap();
    podiumAlert($map);
    recordCreated($map, Character::factory()->create(), '2026-06-10');

    Http::fake();
    $this->artisan('app:post-maintainer-podium')->assertSuccessful();
    $this->artisan('app:post-maintainer-podium', ['--period' => '2026-06'])->assertSuccessful();

    Http::assertSentCount(1);
});

it('posts again after discord_posted_at is cleared', function () {
    $map = podiumMap();
    podiumAlert($map);
    recordCreated($map, Character::factory()->create(), '2026-06-10');

    Http::fake();
    $this->artisan('app:post-maintainer-podium')->assertSuccessful();

    MapMaintainerReport::query()->where('map_id', $map->id)->update(['discord_posted_at' => null]);

    $this->artisan('app:post-maintainer-podium')->assertSuccessful();

    Http::assertSentCount(2);
});

it('rejects a maintainer podium alert that carries a target system', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    $manager = maintainerAlertManager($map);
    actingAs($manager);

    $this->post(route('map-alerts.store'), [
        'map_id' => $map->id,
        'map_webhook_id' => $webhook->id,
        'type' => 'maintainer_podium',
        'target_solarsystem_id' => makeSolarsystem(30009620),
        'is_active' => true,
    ])->assertInvalid(['target_solarsystem_id']);
});

it('lets a manager create a maintainer podium alert with no target, ship, jdc level, or jump count', function () {
    $map = Map::factory()->create();
    $webhook = MapWebhook::factory()->for($map)->create();
    $manager = maintainerAlertManager($map);
    actingAs($manager);

    $this->post(route('map-alerts.store'), [
        'map_id' => $map->id,
        'map_webhook_id' => $webhook->id,
        'type' => 'maintainer_podium',
        'is_active' => true,
    ])->assertRedirect();

    $alert = MapAlert::query()->where('map_id', $map->id)->sole();

    expect($alert->type)->toBe(MapAlertType::MaintainerPodium)
        ->and($alert->target_solarsystem_id)->toBeNull()
        ->and($alert->delivery_type)->toBe(MapAlertDeliveryType::Webhook);
});

it('handles the maintainer podium type in the label and the Discord formatter', function () {
    $map = Map::factory()->create(['name' => 'Formatter Map']);
    $webhook = MapWebhook::factory()->for($map)->create();
    $alert = MapAlert::factory()->maintainerPodium()->create([
        'map_id' => $map->id,
        'map_webhook_id' => $webhook->id,
    ]);

    expect(MapAlertType::MaintainerPodium->label())->toBe('Monthly maintainer podium')
        ->and(app(DiscordAlertFormatter::class)->format($alert))->toContain('monthly maintainer podium');
});

it('refuses to create a maintainer podium alert through the Discord slash command', function () {
    $map = Map::factory()->create();
    $account = DiscordAccount::factory()->create();

    $response = app(CreateDiscordAlertAction::class)->handle(
        $account,
        MapAlertType::MaintainerPodium,
        MapAlertDeliveryType::DiscordDm,
        $map->id,
        null,
        null,
        null,
        null,
        false,
        MapAlertMentionMode::None,
        null,
        null,
        null,
    );

    expect($response)->toBe('Maintainer podium alerts can only be set up in the map settings.');
});

it('continues processing remaining maps when one fails to finalize, and logs the failure', function () {
    Log::spy();

    $failing = podiumMap();
    $ok = podiumMap();
    podiumAlert($ok);
    recordCreated($ok, Character::factory()->create(), '2026-06-10');

    $real = app(FinalizeMaintainerReportAction::class);

    $this->app->bind(FinalizeMaintainerReportAction::class, function () use ($failing, $real) {
        return new class($failing->id, $real)
        {
            public function __construct(private int $failingMapId, private FinalizeMaintainerReportAction $real) {}

            public function handle(Map $map, MaintainerPeriod $period): MapMaintainerReport
            {
                if ($map->id === $this->failingMapId) {
                    throw new RuntimeException('boom');
                }

                return $this->real->handle($map, $period);
            }
        };
    });

    Http::fake();
    $this->artisan('app:post-maintainer-podium')->assertSuccessful();

    Http::assertSentCount(1);
    expect(MapMaintainerReport::query()->where('map_id', $ok->id)->exists())->toBeTrue()
        ->and(MapMaintainerReport::query()->where('map_id', $failing->id)->exists())->toBeFalse();

    Log::shouldHaveReceived('error')->once();
});
