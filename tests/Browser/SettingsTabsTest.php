<?php

declare(strict_types=1);

use App\Models\Character;
use App\Models\Map;
use App\Models\User;

use function Pest\Laravel\actingAs;

it('navigates map Discord tabs through server-provided tabs', function () {
    $map = Map::factory()->create();
    $this->actAsMapOwner($map);

    visit(route('maps.settings.discord.show', $map))
        ->assertSee('Bot alerts')
        ->click('[data-testid="discord-tab-webhooks"]')
        ->assertQueryStringHas('tab', 'webhooks')
        ->assertSee('Webhook alerts')
        ->click('[data-testid="discord-tab-bot"]')
        ->assertQueryStringHas('tab', 'bot')
        ->assertSee('Bot alerts')
        ->assertNoSmoke()
        ->assertNoConsoleLogs();
});

it('navigates account settings tabs through server-provided sections', function () {
    $user = User::factory()->has(Character::factory(['name' => 'Tab Pilot']))->create();
    $user->update(['preferred_character_id' => $user->characters()->value('id')]);
    actingAs($user);

    visit(route('settings.show'))
        ->assertSee('Characters and ESI')
        ->click('[data-testid="settings-tab-discord"]')
        ->assertQueryStringHas('section', 'discord')
        ->assertSee('Not connected')
        ->click('[data-testid="settings-tab-tokens"]')
        ->assertQueryStringHas('section', 'tokens')
        ->assertSee('New token')
        ->click('[data-testid="settings-tab-esi"]')
        ->assertQueryStringHas('section', 'esi')
        ->assertSee('Characters and ESI')
        ->assertNoSmoke()
        ->assertNoConsoleLogs();
});

it('confirms before removing all character permissions', function () {
    $user = User::factory()->has(Character::factory(['name' => 'Scope Pilot']))->create();
    $user->update(['preferred_character_id' => $user->characters()->value('id')]);
    actingAs($user);

    visit(route('settings.show'))
        ->click('button[aria-label="Actions for Scope Pilot"]')
        ->click('Remove all permissions')
        ->assertSee('Remove all permissions for Scope Pilot?')
        ->press('Cancel')
        ->assertDontSee('Remove all permissions for Scope Pilot?')
        ->assertNoSmoke()
        ->assertNoConsoleLogs();
});
