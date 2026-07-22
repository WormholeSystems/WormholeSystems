<?php

declare(strict_types=1);

use App\Enums\Permission;
use App\Models\Character;
use App\Models\DiscordAccount;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\User;
use App\Services\Discord\DiscordAutocomplete;
use App\Services\Discord\DiscordInteractionHandler;
use Discord\Discord;
use Discord\Helpers\RegisteredCommand;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Http;

it('registers the Discord command surface', function () {
    config()->set('services.discord.application_id', 'application-id');
    config()->set('services.discord.bot_token', 'bot-token');
    config()->set('services.discord.test_guild_id', 'guild-id');
    Http::fake(['discord.com/api/v10/*' => Http::response([])]);

    $this->artisan('discord:register-commands')->assertSuccessful();

    Http::assertSent(function ($request): bool {
        $commands = $request->data();
        $alert = collect($commands)->firstWhere('name', 'alert');
        $alerts = collect($commands)->firstWhere('name', 'alerts');
        $dm = collect($alert['options'])->firstWhere('name', 'dm');
        $channel = collect($alert['options'])->firstWhere('name', 'channel');
        $channelProximity = collect($channel['options'])->firstWhere('name', 'proximity');
        $dmJumpRange = collect($dm['options'])->firstWhere('name', 'jump-range');
        $dmKillmail = collect($dm['options'])->firstWhere('name', 'killmail');
        $mention = collect($channelProximity['options'])->firstWhere('name', 'mention');
        $role = collect($channelProximity['options'])->firstWhere('name', 'role');

        return $request->method() === 'PUT'
            && str_ends_with($request->url(), '/applications/application-id/guilds/guild-id/commands')
            && collect($commands)->pluck('name')->all() === ['account', 'alert', 'alerts', 'route']
            && $dm['type'] === 2
            && $channel['type'] === 2
            && collect($dm['options'])->pluck('name')->all() === ['proximity', 'jump-range', 'killmail']
            && collect($channel['options'])->pluck('name')->all() === ['proximity', 'jump-range', 'killmail']
            && collect(collect($dm['options'])->firstWhere('name', 'proximity')['options'])->pluck('name')->all() === ['map', 'system', 'jumps']
            && collect($dmJumpRange['options'])->pluck('name')->all() === ['map', 'system', 'ship', 'jdc', 'highsec']
            && collect($dmKillmail['options'])->pluck('name')->all() === ['map', 'jumps']
            && collect($alerts['options'])->pluck('name')->all() === ['list', 'map', 'enable', 'disable', 'remove']
            && collect($mention['choices'])->pluck('value')->all() === ['none', 'creator', 'role', 'everyone']
            && $mention['required'] === true
            && $role['type'] === 8
            && $role['required'] === false;
    });
});

it('refuses to start the bot without credentials', function () {
    config()->set('services.discord.bot_token');

    $this->artisan('discord:listen')->assertFailed();
});

it('requires an explicit global flag when no test guild is configured', function () {
    config()->set('services.discord.application_id', 'application-id');
    config()->set('services.discord.bot_token', 'bot-token');
    config()->set('services.discord.test_guild_id');
    Http::fake();

    $this->artisan('discord:register-commands')->assertFailed();
    Http::assertNothingSent();
});

it('creates a distinct restart signal for every request', function () {
    $this->artisan('discord:restart')->assertSuccessful();
    $firstSignal = Cache::get('discord.restart');

    $this->artisan('discord:restart')->assertSuccessful();

    expect($firstSignal)->toBeString()
        ->and(Cache::get('discord.restart'))->toBeString()->not->toBe($firstSignal);
});

it('returns only maps accessible to the linked account for autocomplete', function () {
    $accessibleMap = Map::factory()->create(['name' => 'Alpha Chain']);
    Map::factory()->create(['name' => 'Hidden Chain']);
    $user = User::factory()->create();
    $character = Character::factory()->for($user)->create();
    MapAccess::factory(['permission' => Permission::Viewer])->for($accessibleMap)->for($character, 'accessible')->create();
    $account = DiscordAccount::factory()->for($user)->create();
    $account->setRelation('user', $user->refresh()->load('characters'));

    $choices = app(DiscordAutocomplete::class)->maps($account, 'Chain');

    expect($choices)->toBe([
        ['name' => 'Alpha Chain', 'value' => (string) $accessibleMap->id],
    ]);
});

it('registers a direct autocomplete interaction listener', function () {
    $accountCommand = Mockery::mock(RegisteredCommand::class);
    $alertCommand = Mockery::mock(RegisteredCommand::class);
    $alertsCommand = Mockery::mock(RegisteredCommand::class);
    $routeCommand = Mockery::mock(RegisteredCommand::class);

    $discord = Mockery::mock(Discord::class);
    $discord->shouldReceive('listenCommand')->once()->with('account', Mockery::type('callable'))->andReturn($accountCommand);
    $discord->shouldReceive('listenCommand')->once()->with('alert', Mockery::type('callable'))->andReturn($alertCommand);
    $discord->shouldReceive('listenCommand')->once()->with('alerts', Mockery::type('callable'))->andReturn($alertsCommand);
    $discord->shouldReceive('listenCommand')->once()->with('route', Mockery::type('callable'))->andReturn($routeCommand);
    $discord->shouldReceive('on')->once()->with('INTERACTION_CREATE', Mockery::type('callable'));

    app(DiscordInteractionHandler::class)->listen($discord);
});
