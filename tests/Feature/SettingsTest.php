<?php

declare(strict_types=1);

use App\Models\Character;
use App\Models\DiscordAccount;
use App\Models\User;
use Illuminate\Support\Facades\Cache;
use Inertia\Testing\AssertableInertia as Assert;

it('renders the consolidated account settings dashboard', function () {
    config()->set('services.discord.application_id', '1529157346406826054');
    $user = User::factory()->create();
    $character = Character::factory()->for($user)->create();
    $user->update(['preferred_character_id' => $character->id]);
    DiscordAccount::factory()->for($user)->create();
    $user->createToken('Mapper API');
    $user->refresh()->load('characters');

    $this->actingAs($user)->get(route('settings.show'))
        ->assertSuccessful()
        ->assertInertia(fn (Assert $page) => $page
            ->component('settings/Show')
            ->where('section', 'esi')
            ->has('characters', 1)
            ->where('characters.0.is_preferred', true)
            ->where('discordAccount.username', $user->discordAccount->username)
            ->where('discordInviteUrl', 'https://discord.com/oauth2/authorize?client_id=1529157346406826054&scope=bot%20applications.commands&permissions=19456&integration_type=0')
            ->has('discordAlerts', 0)
            ->has('tokens', 1));
});

it('keeps a pending Discord account available for confirmation', function () {
    $user = User::factory()->has(Character::factory())->create();
    $user->update(['preferred_character_id' => $user->characters()->value('id')]);
    $user->refresh();

    Cache::put('discord.link.pending', [
        'id' => '223456789012345678',
        'username' => 'pathfinder',
        'global_name' => 'Pathfinder',
        'avatar' => null,
    ]);

    $this->actingAs($user)
        ->withSession(['pending_discord_link_token' => 'pending'])
        ->get(route('settings.show'))
        ->assertSuccessful()
        ->assertInertia(fn (Assert $page) => $page
            ->where('pendingDiscordAccount.username', 'pathfinder')
            ->where('pendingDiscordAccount.global_name', 'Pathfinder')
            ->etc());
});

it('redirects legacy account pages into settings', function (string $routeName, string $section) {
    $user = User::factory()->create();

    $this->actingAs($user)->get(route($routeName))
        ->assertRedirect(route('settings.show', ['section' => $section]));
})->with([
    ['scopes.index', 'esi'],
    ['tokens.index', 'tokens'],
]);

it('renders account settings for each supported section URL', function (string $section) {
    $user = User::factory()->has(Character::factory())->create();
    $user->update(['preferred_character_id' => $user->characters()->value('id')]);

    $this->actingAs($user)
        ->get(route('settings.show', ['section' => $section]))
        ->assertSuccessful()
        ->assertInertia(fn (Assert $page) => $page
            ->component('settings/Show')
            ->where('section', $section)
            ->has('characters', 1)
            ->has('discordAlerts')
            ->has('tokens'));
})->with(['esi', 'discord', 'tokens']);

it('defaults an unsupported account settings section', function () {
    $user = User::factory()->has(Character::factory())->create();
    $user->update(['preferred_character_id' => $user->characters()->value('id')]);

    $this->actingAs($user)
        ->get(route('settings.show', ['section' => 'unknown']))
        ->assertSuccessful()
        ->assertInertia(fn (Assert $page) => $page->where('section', 'esi')->etc());
});
