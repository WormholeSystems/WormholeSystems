<?php

declare(strict_types=1);

use App\Enums\MapAlertDisabledReason;
use App\Enums\MapAlertMentionMode;
use App\Enums\Permission;
use App\Jobs\MapAlerts\EvaluateMapAlertsJob;
use App\Models\Character;
use App\Models\DiscordAccount;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapAlert;
use App\Models\MapAlertDelivery;
use App\Models\MapSolarsystem;
use App\Models\User;
use Illuminate\Http\Client\RequestException;
use Illuminate\Support\Facades\Http;

beforeEach(function (): void {
    config()->set('services.discord.bot_token', 'bot-token');
});

it('delivers a direct message once for a newly placed system in range', function (): void {
    Http::fake([
        'discord.com/api/v10/users/@me/channels' => Http::response(['id' => 'dm-channel']),
        'discord.com/api/v10/channels/dm-channel/messages' => Http::response(['id' => 'message']),
    ]);
    $systemId = makeSolarsystem(30009990);
    $map = Map::factory()->create();
    $user = User::factory()->create();
    grantMapViewAccess($map, $user);
    DiscordAccount::factory()->for($user)->create(['discord_user_id' => 'dm-user']);
    $placement = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $systemId]);
    $alert = MapAlert::factory()->discordDm()->create([
        'map_id' => $map->id,
        'created_by_user_id' => $user->id,
        'target_solarsystem_id' => $systemId,
    ]);

    app()->call([new EvaluateMapAlertsJob($placement->id), 'handle']);

    Http::assertSentCount(2);
    Http::assertSent(fn ($request): bool => str_ends_with($request->url(), '/users/@me/channels')
        && $request['recipient_id'] === 'dm-user');
    Http::assertSent(fn ($request): bool => str_ends_with($request->url(), '/channels/dm-channel/messages')
        && $request['nonce'] === mb_substr(hash('sha256', $alert->id.':'.$placement->id), 0, 25)
        && $request['enforce_nonce'] === true
        && $request['embeds'][0]['url'] === route('maps.show', $map));
    expect($alert->refresh()->last_fired_at)->not->toBeNull()
        ->and($alert->deliveries)->toHaveCount(1)
        ->and($alert->deliveries->sole()->delivered_at)->not->toBeNull();

    Http::fake();
    app()->call([new EvaluateMapAlertsJob($placement->id), 'handle']);
    Http::assertNothingSent();
});

it('reclaims a stale unfinished delivery reservation', function (): void {
    Http::fake(['discord.com/api/v10/channels/reclaimed/messages' => Http::response(['id' => 'message'])]);
    $systemId = makeSolarsystem(30010503);
    $map = Map::factory()->create();
    $placement = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $systemId]);
    $alert = MapAlert::factory()->discordChannel('guild', 'reclaimed')->create([
        'map_id' => $map->id,
        'target_solarsystem_id' => $systemId,
    ]);
    $reservation = MapAlertDelivery::query()->create([
        'map_alert_id' => $alert->id,
        'map_solarsystem_id' => $placement->id,
    ]);
    $reservation->timestamps = false;
    $reservation->update([
        'created_at' => now()->subMinutes(11),
        'updated_at' => now()->subMinutes(11),
    ]);

    app()->call([new EvaluateMapAlertsJob($placement->id), 'handle']);

    Http::assertSentCount(1);
    expect($reservation->refresh()->delivered_at)->not->toBeNull()
        ->and($alert->refresh()->last_fired_at)->not->toBeNull();
});

it('disables an alert whose payload Discord rejects', function (): void {
    Http::fake(['discord.com/api/v10/channels/rejected/messages' => Http::response([], 400)]);
    $systemId = makeSolarsystem(30010504);
    $map = Map::factory()->create();
    $placement = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $systemId]);
    $alert = MapAlert::factory()->discordChannel('guild', 'rejected')->create([
        'map_id' => $map->id,
        'target_solarsystem_id' => $systemId,
    ]);

    app()->call([new EvaluateMapAlertsJob($placement->id), 'handle']);

    expect($alert->refresh()->is_active)->toBeFalse()
        ->and($alert->disabled_reason)->toBe(MapAlertDisabledReason::DeliveryFailed)
        ->and($alert->deliveries)->toHaveCount(1);
});

it('retries a Discord rate limit using Retry-After and then succeeds', function (): void {
    Http::fake([
        'discord.com/api/v10/channels/rate-limited/messages' => Http::sequence()
            ->push([], 429, ['Retry-After' => '0.001'])
            ->push(['id' => 'message']),
    ]);
    $systemId = makeSolarsystem(30009989);
    $map = Map::factory()->create();
    $placement = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $systemId]);
    $alert = MapAlert::factory()->discordChannel('guild', 'rate-limited')->create([
        'map_id' => $map->id,
        'target_solarsystem_id' => $systemId,
    ]);

    app()->call([new EvaluateMapAlertsJob($placement->id), 'handle']);

    Http::assertSentCount(2);
    expect($alert->refresh()->last_fired_at)->not->toBeNull()
        ->and($alert->deliveries)->toHaveCount(1);
});

it('disables and retains a direct message alert after map access is revoked', function (): void {
    Http::fake();
    $systemId = makeSolarsystem(30009991);
    $map = Map::factory()->create();
    $user = User::factory()->create();
    Character::factory()->for($user)->create();
    DiscordAccount::factory()->for($user)->create();
    $placement = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $systemId]);
    $alert = MapAlert::factory()->discordDm()->create([
        'map_id' => $map->id,
        'created_by_user_id' => $user->id,
        'target_solarsystem_id' => $systemId,
    ]);

    app()->call([new EvaluateMapAlertsJob($placement->id), 'handle']);

    expect($alert->refresh()->is_active)->toBeFalse()
        ->and($alert->disabled_at)->not->toBeNull()
        ->and($alert->disabled_reason)->toBe(MapAlertDisabledReason::AccessRevoked)
        ->and(MapAlert::query()->find($alert->id))->not->toBeNull();
    Http::assertNothingSent();
});

it('disables and retains a direct message alert when Discord is unlinked', function (): void {
    Http::fake();
    $systemId = makeSolarsystem(30009992);
    $map = Map::factory()->create();
    $user = User::factory()->create();
    grantMapViewAccess($map, $user);
    $placement = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $systemId]);
    $alert = MapAlert::factory()->discordDm()->create([
        'map_id' => $map->id,
        'created_by_user_id' => $user->id,
        'target_solarsystem_id' => $systemId,
    ]);

    app()->call([new EvaluateMapAlertsJob($placement->id), 'handle']);

    expect($alert->refresh()->is_active)->toBeFalse()
        ->and($alert->disabled_at)->not->toBeNull()
        ->and($alert->disabled_reason)->toBe(MapAlertDisabledReason::DiscordAccountDisconnected)
        ->and(MapAlert::query()->find($alert->id))->not->toBeNull();
    Http::assertNothingSent();
});

it('keeps delivering a shared channel alert after its creator loses access', function (): void {
    Http::fake(['discord.com/api/v10/channels/shared-channel/messages' => Http::response(['id' => 'message'])]);
    $systemId = makeSolarsystem(30009993);
    $map = Map::factory()->create();
    $user = User::factory()->create();
    Character::factory()->for($user)->create();
    $placement = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $systemId]);
    $alert = MapAlert::factory()->discordChannel('guild', 'shared-channel')->create([
        'map_id' => $map->id,
        'created_by_user_id' => $user->id,
        'target_solarsystem_id' => $systemId,
    ]);

    app()->call([new EvaluateMapAlertsJob($placement->id), 'handle']);

    expect($alert->refresh()->is_active)->toBeTrue()
        ->and($alert->last_fired_at)->not->toBeNull();
    Http::assertSent(fn ($request): bool => str_ends_with($request->url(), '/channels/shared-channel/messages'));
});

it('delivers a channel alert without mentions', function (): void {
    Http::fake(['discord.com/api/v10/channels/alert-channel/messages' => Http::response(['id' => 'message'])]);
    $systemId = makeSolarsystem(30009994);
    $map = Map::factory()->create();
    $placement = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $systemId]);
    MapAlert::factory()->discordChannel('guild', 'alert-channel')->create([
        'map_id' => $map->id,
        'created_by_user_id' => null,
        'target_solarsystem_id' => $systemId,
        'mention_mode' => MapAlertMentionMode::None,
    ]);

    app()->call([new EvaluateMapAlertsJob($placement->id), 'handle']);

    Http::assertSent(fn ($request): bool => $request['content'] === null
        && $request['allowed_mentions'] === ['parse' => [], 'users' => [], 'roles' => []]);
});

it('mentions only the creator for a creator mention', function (): void {
    Http::fake(['discord.com/api/v10/channels/alert-channel/messages' => Http::response(['id' => 'message'])]);
    $systemId = makeSolarsystem(30009995);
    $map = Map::factory()->create();
    $user = User::factory()->create();
    DiscordAccount::factory()->for($user)->create(['discord_user_id' => '423456789012345678']);
    $placement = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $systemId]);
    MapAlert::factory()->discordChannel('guild', 'alert-channel')->mentionsCreator()->create([
        'map_id' => $map->id,
        'created_by_user_id' => $user->id,
        'target_solarsystem_id' => $systemId,
    ]);

    app()->call([new EvaluateMapAlertsJob($placement->id), 'handle']);

    Http::assertSent(fn ($request): bool => $request['content'] === '<@423456789012345678>'
        && $request['allowed_mentions'] === [
            'parse' => [],
            'users' => ['423456789012345678'],
            'roles' => [],
        ]);
});

it('mentions only the selected role for a role mention', function (): void {
    Http::fake(['discord.com/api/v10/channels/alert-channel/messages' => Http::response(['id' => 'message'])]);
    $systemId = makeSolarsystem(30009996);
    $map = Map::factory()->create();
    $placement = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $systemId]);
    MapAlert::factory()->discordChannel('guild', 'alert-channel')->mentionsRole('523456789012345678')->create([
        'map_id' => $map->id,
        'created_by_user_id' => null,
        'target_solarsystem_id' => $systemId,
    ]);

    app()->call([new EvaluateMapAlertsJob($placement->id), 'handle']);

    Http::assertSent(fn ($request): bool => $request['content'] === '<@&523456789012345678>'
        && $request['allowed_mentions'] === [
            'parse' => [],
            'users' => [],
            'roles' => ['523456789012345678'],
        ]);
});

it('pings everyone when the selected role is the guild everyone role', function (): void {
    Http::fake(['discord.com/api/v10/channels/alert-channel/messages' => Http::response(['id' => 'message'])]);
    $systemId = makeSolarsystem(30009997);
    $map = Map::factory()->create();
    $placement = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $systemId]);
    MapAlert::factory()->discordChannel('623456789012345678', 'alert-channel')->mentionsRole('623456789012345678')->create([
        'map_id' => $map->id,
        'created_by_user_id' => null,
        'target_solarsystem_id' => $systemId,
    ]);

    app()->call([new EvaluateMapAlertsJob($placement->id), 'handle']);

    Http::assertSent(fn ($request): bool => $request['content'] === '@everyone'
        && $request['allowed_mentions'] === [
            'parse' => ['everyone'],
            'users' => [],
            'roles' => [],
        ]);
});

it('pings everyone for an everyone mention mode', function (): void {
    Http::fake(['discord.com/api/v10/channels/alert-channel/messages' => Http::response(['id' => 'message'])]);
    $systemId = makeSolarsystem(30009998);
    $map = Map::factory()->create();
    $placement = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $systemId]);
    MapAlert::factory()->discordChannel('guild', 'alert-channel')->create([
        'map_id' => $map->id,
        'created_by_user_id' => null,
        'target_solarsystem_id' => $systemId,
        'mention_mode' => MapAlertMentionMode::Everyone,
    ]);

    app()->call([new EvaluateMapAlertsJob($placement->id), 'handle']);

    Http::assertSent(fn ($request): bool => $request['content'] === '@everyone'
        && $request['allowed_mentions'] === [
            'parse' => ['everyone'],
            'users' => [],
            'roles' => [],
        ]);
});

it('disables a creator mention channel alert whose creator unlinked Discord', function (): void {
    Http::fake();
    $systemId = makeSolarsystem(30010001);
    $map = Map::factory()->create();
    $user = User::factory()->create();
    $placement = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $systemId]);
    $alert = MapAlert::factory()->discordChannel('guild', 'alert-channel')->mentionsCreator()->create([
        'map_id' => $map->id,
        'created_by_user_id' => $user->id,
        'target_solarsystem_id' => $systemId,
    ]);

    app()->call([new EvaluateMapAlertsJob($placement->id), 'handle']);

    Http::assertNothingSent();
    expect($alert->refresh()->is_active)->toBeFalse()
        ->and($alert->disabled_reason)->toBe(MapAlertDisabledReason::DiscordAccountDisconnected);
});

it('does not disable alerts when the bot token is missing', function (): void {
    config()->set('services.discord.bot_token', null);
    Http::fake();
    $systemId = makeSolarsystem(30010002);
    $map = Map::factory()->create();
    $user = User::factory()->create();
    grantMapViewAccess($map, $user);
    DiscordAccount::factory()->for($user)->create(['discord_user_id' => 'dm-user']);
    $placement = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $systemId]);
    $alert = MapAlert::factory()->discordDm()->create([
        'map_id' => $map->id,
        'created_by_user_id' => $user->id,
        'target_solarsystem_id' => $systemId,
    ]);

    expect(fn () => app()->call([new EvaluateMapAlertsJob($placement->id), 'handle']))
        ->toThrow(App\Services\Discord\DiscordConfigurationException::class);

    expect($alert->refresh()->is_active)->toBeTrue()
        ->and($alert->disabled_reason)->toBeNull()
        ->and($alert->deliveries()->count())->toBe(0);
});

it('disables a permanently unavailable Discord destination and retains its reservation', function (int $status): void {
    Http::fake([
        'discord.com/api/v10/channels/unavailable/messages' => Http::response([], $status),
    ]);
    $systemId = makeSolarsystem(30009997 + $status);
    $map = Map::factory()->create();
    $placement = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $systemId]);
    $alert = MapAlert::factory()->discordChannel('guild', 'unavailable')->create([
        'map_id' => $map->id,
        'target_solarsystem_id' => $systemId,
    ]);

    app()->call([new EvaluateMapAlertsJob($placement->id), 'handle']);

    expect($alert->refresh()->is_active)->toBeFalse()
        ->and($alert->disabled_reason)->toBe(MapAlertDisabledReason::DiscordDestinationUnavailable)
        ->and($alert->deliveries)->toHaveCount(1)
        ->and($alert->events()->latest('id')->firstOrFail()->reason)->toBe('Discord returned HTTP '.$status.' for the alert destination.');
    Http::assertSentCount(1);
})->with([403, 404]);

it('does not retry a server failure and continues before making the job retryable', function (): void {
    Http::fake(function ($request) {
        if (str_ends_with($request->url(), '/channels/failing/messages')) {
            return Http::response([], 500);
        }

        return Http::response(['id' => 'message']);
    });
    $systemId = makeSolarsystem(30010500);
    $map = Map::factory()->create();
    $placement = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $systemId]);
    $failedAlert = MapAlert::factory()->discordChannel('guild', 'failing')->create([
        'map_id' => $map->id,
        'target_solarsystem_id' => $systemId,
    ]);
    $successfulAlert = MapAlert::factory()->discordChannel('guild', 'working')->create([
        'map_id' => $map->id,
        'target_solarsystem_id' => $systemId,
    ]);

    expect(fn () => app()->call([new EvaluateMapAlertsJob($placement->id), 'handle']))
        ->toThrow(RequestException::class);

    Http::assertSentCount(2);
    expect($failedAlert->refresh()->deliveries)->toHaveCount(0)
        ->and($failedAlert->is_active)->toBeTrue()
        ->and($successfulAlert->refresh()->last_fired_at)->not->toBeNull()
        ->and($successfulAlert->deliveries)->toHaveCount(1)
        ->and((new EvaluateMapAlertsJob($placement->id))->tries)->toBeGreaterThan(1);
});

it('disables and retains a malformed alert without throwing', function (): void {
    Http::fake();
    $systemId = makeSolarsystem(30010501);
    $map = Map::factory()->create();
    $placement = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $systemId]);
    $alert = MapAlert::factory()->discordDm()->create([
        'map_id' => $map->id,
        'created_by_user_id' => null,
        'target_solarsystem_id' => $systemId,
    ]);

    app()->call([new EvaluateMapAlertsJob($placement->id), 'handle']);

    expect($alert->refresh()->is_active)->toBeFalse()
        ->and($alert->disabled_reason)->toBe(MapAlertDisabledReason::DeliveryFailed)
        ->and($alert->deliveries)->toHaveCount(1)
        ->and($alert->events()->latest('id')->firstOrFail()->reason)->toBe('Discord direct message alert does not have a creator.');
    Http::assertNothingSent();
});

it('continues delivering after one Discord destination is permanently unavailable', function (): void {
    Http::fake(function ($request) {
        if (str_ends_with($request->url(), '/users/@me/channels') && $request['recipient_id'] === 'failed-user') {
            return Http::response([], 403);
        }

        if (str_ends_with($request->url(), '/users/@me/channels')) {
            return Http::response(['id' => 'working-dm']);
        }

        return Http::response(['id' => 'message']);
    });
    $systemId = makeSolarsystem(30010502);
    $map = Map::factory()->create();
    $placement = MapSolarsystem::factory()->for($map)->create(['solarsystem_id' => $systemId]);
    $alerts = collect();

    foreach (['failed-user', 'working-user'] as $discordUserId) {
        $user = User::factory()->create();
        grantMapViewAccess($map, $user);
        DiscordAccount::factory()->for($user)->create(['discord_user_id' => $discordUserId]);
        $alerts->put($discordUserId, MapAlert::factory()->discordDm()->create([
            'map_id' => $map->id,
            'created_by_user_id' => $user->id,
            'target_solarsystem_id' => $systemId,
        ]));
    }

    app()->call([new EvaluateMapAlertsJob($placement->id), 'handle']);

    expect($alerts->get('failed-user')->refresh()->deliveries)->toHaveCount(1)
        ->and($alerts->get('failed-user')->disabled_reason)->toBe(MapAlertDisabledReason::DiscordDestinationUnavailable)
        ->and($alerts->get('working-user')->refresh()->last_fired_at)->not->toBeNull()
        ->and($alerts->get('working-user')->deliveries)->toHaveCount(1);
});

function grantMapViewAccess(Map $map, User $user): void
{
    $character = Character::factory()->for($user)->create();
    MapAccess::factory(['permission' => Permission::Viewer])->for($map)->for($character, 'accessible')->create();
}
