<?php

declare(strict_types=1);

use App\Enums\MapAlertDisabledReason;
use App\Enums\MapAlertEventAction;
use App\Enums\MapAlertMentionMode;
use App\Models\DiscordAccount;
use App\Models\Map;
use App\Models\MapAlert;
use App\Models\User;
use Illuminate\Support\Facades\Cache;
use Laravel\Socialite\Facades\Socialite;
use Laravel\Socialite\Two\InvalidStateException;
use SocialiteProviders\Manager\OAuth2\User as SocialiteUser;

it('redirects to Discord to authorize', function () {
    config()->set('services.discord', [
        'client_id' => 'client-id',
        'client_secret' => 'secret',
        'redirect' => 'https://wormholesystems.test/discord/callback',
    ]);
    $user = User::factory()->create();

    $this->actingAs($user)->get(route('discord.connect'))
        ->assertRedirect()
        ->assertRedirectContains('discord.com')
        ->assertRedirectContains('prompt=consent');
});

it('links a Discord account through OAuth', function () {
    $identity = (new SocialiteUser)->setRaw([
        'id' => '123456789012345678',
        'username' => 'mapper',
        'global_name' => 'Mapper',
        'avatar' => 'avatar-hash',
    ])->map(['id' => '123456789012345678', 'nickname' => 'mapper']);
    Socialite::shouldReceive('driver->user')->andReturn($identity);
    $user = User::factory()->create();

    $response = $this->actingAs($user)->get(route('discord.callback', ['state' => 'state', 'code' => 'code']));

    $response->assertRedirect(route('settings.show', ['section' => 'discord']));
    $this->assertDatabaseHas(DiscordAccount::class, [
        'user_id' => $user->id,
        'discord_user_id' => '123456789012345678',
        'display_name' => 'Mapper',
    ]);
});

it('rejects an invalid OAuth state', function () {
    Socialite::shouldReceive('driver->user')->andThrow(new InvalidStateException);
    $user = User::factory()->create();

    $this->actingAs($user)->get(route('discord.callback', ['state' => 'wrong', 'code' => 'code']))
        ->assertForbidden();
});

it('consumes a bot-generated account link only when confirmed', function () {
    $user = User::factory()->create();
    Cache::put('discord.link.one-time', [
        'id' => '223456789012345678',
        'username' => 'pathfinder',
        'global_name' => null,
        'avatar' => null,
    ]);

    $this->actingAs($user)->get(route('discord.link', 'one-time'))->assertRedirect(route('settings.show', ['section' => 'discord']));
    $this->assertDatabaseMissing(DiscordAccount::class, ['user_id' => $user->id]);
    $this->actingAs($user)->get(route('discord.link', 'one-time'))->assertRedirect();
    $this->actingAs($user)->post(route('discord.link.confirm'))->assertRedirect(route('settings.show', ['section' => 'discord']));
    $this->assertDatabaseHas(DiscordAccount::class, ['user_id' => $user->id, 'discord_user_id' => '223456789012345678']);
    $this->actingAs($user)->get(route('discord.link', 'one-time'))->assertNotFound();
});

it('does not allow one Discord identity on two users', function () {
    DiscordAccount::factory()->create(['discord_user_id' => '323456789012345678']);
    $user = User::factory()->create();
    Cache::put('discord.link.conflict', [
        'id' => '323456789012345678',
        'username' => 'duplicate',
        'global_name' => null,
        'avatar' => null,
    ]);

    $this->actingAs($user)->get(route('discord.link', 'conflict'))->assertRedirect();
    $this->actingAs($user)->post(route('discord.link.confirm'))->assertStatus(409);
});

it('disables only active alerts that depend on the unlinked Discord identity', function () {
    $user = User::factory()->create();
    DiscordAccount::factory()->for($user)->create();
    $map = Map::factory()->create();
    $targetId = makeSolarsystem(30009510);
    $directMessage = MapAlert::factory()->discordDm()->create([
        'map_id' => $map->id,
        'created_by_user_id' => $user->id,
        'target_solarsystem_id' => $targetId,
    ]);
    $creatorMention = MapAlert::factory()->discordChannel()->mentionsCreator()->create([
        'map_id' => $map->id,
        'created_by_user_id' => $user->id,
        'target_solarsystem_id' => $targetId,
    ]);
    $roleMention = MapAlert::factory()->discordChannel()->mentionsRole()->create([
        'map_id' => $map->id,
        'created_by_user_id' => $user->id,
        'target_solarsystem_id' => $targetId,
    ]);
    $noMention = MapAlert::factory()->discordChannel()->create([
        'map_id' => $map->id,
        'created_by_user_id' => $user->id,
        'mention_mode' => MapAlertMentionMode::None,
        'target_solarsystem_id' => $targetId,
    ]);
    $inactive = MapAlert::factory()->discordDm()->disabled()->create([
        'map_id' => $map->id,
        'created_by_user_id' => $user->id,
        'target_solarsystem_id' => $targetId,
    ]);

    $this->actingAs($user)->delete(route('discord.destroy'))->assertRedirect();

    expect($user->discordAccount()->exists())->toBeFalse()
        ->and($directMessage->refresh()->disabled_reason)->toBe(MapAlertDisabledReason::DiscordAccountDisconnected)
        ->and($creatorMention->refresh()->disabled_reason)->toBe(MapAlertDisabledReason::DiscordAccountDisconnected)
        ->and($roleMention->refresh()->is_active)->toBeTrue()
        ->and($noMention->refresh()->is_active)->toBeTrue()
        ->and($inactive->refresh()->disabled_reason)->toBe(MapAlertDisabledReason::Manual)
        ->and($directMessage->events()->where('action', MapAlertEventAction::Disabled)->count())->toBe(1)
        ->and($creatorMention->events()->where('action', MapAlertEventAction::Disabled)->count())->toBe(1)
        ->and($roleMention->events()->count())->toBe(0)
        ->and($noMention->events()->count())->toBe(0)
        ->and($inactive->events()->count())->toBe(0);
});

it('handles a cancelled Discord authorization gracefully', function () {
    $user = User::factory()->create();

    $this->actingAs($user)->get(route('discord.callback', ['error' => 'access_denied']))
        ->assertRedirect(route('settings.show', ['section' => 'discord']));

    $this->assertDatabaseMissing(DiscordAccount::class, ['user_id' => $user->id]);
});
