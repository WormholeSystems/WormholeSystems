<?php

declare(strict_types=1);

use App\Enums\MapAlertDeliveryType;
use App\Enums\MapAlertDisabledReason;
use App\Enums\MapAlertEventAction;
use App\Enums\Permission;
use App\Http\Resources\MapAlertEventResource;
use App\Http\Resources\MapAlertResource;
use App\Models\Character;
use App\Models\DiscordAccount;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapAlert;
use App\Models\MapAlertEvent;
use App\Models\User;
use App\Services\MapAlerts\MapAlertLifecycle;
use Illuminate\Http\Request;
use Inertia\Testing\AssertableInertia as Assert;

use function Pest\Laravel\actingAs;

it('shows managers private bot alerts and recent lifecycle history without Discord identifiers', function () {
    $map = Map::factory()->create();
    $manager = User::factory()->ownsMap($map)->create();
    $manager->update(['preferred_character_id' => $manager->characters->first()->id]);
    $manager->refresh();
    $creator = User::factory()->has(Character::factory(['name' => 'Alert Creator']))->create();
    $targetId = makeSolarsystem(30009501);
    $alert = MapAlert::factory()->discordDm()->create([
        'map_id' => $map->id,
        'created_by_user_id' => $creator->id,
        'target_solarsystem_id' => $targetId,
    ]);
    MapAlertEvent::factory()->create([
        'map_alert_id' => null,
        'map_id' => $map->id,
        'actor_user_id' => $manager->id,
        'action' => 'removed',
        'snapshot' => [
            'delivery_type' => 'discord_channel',
            'discord_channel_id' => '123456789012345678',
            'type' => 'proximity',
        ],
        'reason' => 'Security cleanup',
    ]);

    actingAs($manager)
        ->get(route('maps.settings.discord.show', $map))
        ->assertSuccessful()
        ->assertInertia(fn (Assert $page) => $page
            ->where('tab', 'bot')
            ->has('alerts', 0)
            ->has('botAlerts', 1)
            ->where('botAlerts.0.id', $alert->id)
            ->where('botAlerts.0.creator_name', 'Alert Creator')
            ->where('botAlerts.0.destination_summary', 'Private Discord message')
            ->missing('botAlerts.0.discord_channel_id')
            ->has('alertEvents', 1)
            ->where('alertEvents.0.action', 'removed')
            ->where('alertEvents.0.reason', 'Security cleanup')
            ->missing('alertEvents.0.snapshot')
            ->etc());

    actingAs($manager)
        ->get(route('maps.settings.discord.show', [$map, 'tab' => 'webhooks']))
        ->assertSuccessful()
        ->assertInertia(fn (Assert $page) => $page->where('tab', 'webhooks')->etc());
});

it('forbids non-managers from viewing or changing managed alerts', function () {
    $map = Map::factory()->create();
    $member = User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Member])->for($map)))
        ->create();
    $alert = MapAlert::factory()->discordDm()->create([
        'map_id' => $map->id,
        'target_solarsystem_id' => makeSolarsystem(30009502),
    ]);

    actingAs($member)->get(route('maps.settings.discord.show', $map))->assertForbidden();
    actingAs($member)->put(route('map-alerts.state.update', $alert), ['enabled' => false])->assertForbidden();

    expect($alert->refresh()->is_active)->toBeTrue();
});

it('changes bot alert state through the lifecycle and records the manual reason', function () {
    $map = Map::factory()->create();
    $manager = User::factory()->ownsMap($map)->create();
    DiscordAccount::factory()->for($manager)->create();
    $alert = MapAlert::factory()->discordDm()->create([
        'map_id' => $map->id,
        'created_by_user_id' => $manager->id,
        'target_solarsystem_id' => makeSolarsystem(30009503),
    ]);

    expect($manager->can('update', $alert))->toBeTrue();

    actingAs($manager)
        ->put(route('map-alerts.state.update', $alert), ['enabled' => false])
        ->assertRedirect();

    $event = MapAlertEvent::query()->where('map_alert_id', $alert->id)->sole();

    expect($alert->refresh()->is_active)->toBeFalse()
        ->and($alert->disabled_reason)->toBe(MapAlertDisabledReason::Manual)
        ->and($event->action)->toBe(MapAlertEventAction::Disabled)
        ->and($event->actor_user_id)->toBe($manager->id);

    actingAs($manager)
        ->put(route('map-alerts.state.update', $alert), ['enabled' => true])
        ->assertRedirect();

    expect($alert->refresh()->is_active)->toBeTrue()
        ->and($alert->disabled_reason)->toBeNull()
        ->and(MapAlertEvent::query()->where('map_alert_id', $alert->id)->where('action', 'enabled')->exists())->toBeTrue();
});

it('does not duplicate lifecycle events for the same requested state reason and actor', function () {
    $actor = User::factory()->create();
    $alert = MapAlert::factory()->discordDm()->create([
        'created_by_user_id' => $actor->id,
        'target_solarsystem_id' => makeSolarsystem(30009509),
    ]);
    $lifecycle = app(MapAlertLifecycle::class);

    $lifecycle->disable($alert, $actor, MapAlertDisabledReason::Manual);
    $disabledAt = $alert->refresh()->disabled_at;
    $lifecycle->disable($alert, $actor, MapAlertDisabledReason::Manual);

    expect($alert->refresh()->disabled_at)->toEqual($disabledAt)
        ->and($alert->events()->where('action', MapAlertEventAction::Disabled)->count())->toBe(1);

    $lifecycle->enable($alert, $actor);
    $lifecycle->enable($alert, $actor);

    expect($alert->events()->where('action', MapAlertEventAction::Enabled)->count())->toBe(1);
});

it('serializes alerts and actor snapshots safely after users or characters are deleted', function () {
    $creator = User::factory()->create(['name' => 'Account Fallback']);
    $alert = MapAlert::factory()->discordDm()->create([
        'created_by_user_id' => $creator->id,
        'target_solarsystem_id' => makeSolarsystem(30009511),
    ]);
    app(MapAlertLifecycle::class)->disable($alert, $creator, MapAlertDisabledReason::Manual);
    $event = $alert->events()->sole();

    expect($creator->alertDisplayName())->toBe('Account Fallback')
        ->and($event->actor_name)->toBe('Account Fallback');

    $creator->delete();
    $alert->refresh()->load(['creator.preferredCharacter', 'creator.characters']);
    $event->refresh()->load(['actor.preferredCharacter', 'actor.characters']);

    $alertData = (new MapAlertResource($alert))->toArray(Request::create('/'));
    $eventData = (new MapAlertEventResource($event))->toArray(Request::create('/'));

    expect($alertData['creator_name'])->toBeNull()
        ->and($eventData['actor_name'])->toBe('Account Fallback')
        ->and($eventData['action'])->toBe(MapAlertEventAction::Disabled->value);
});

it('allows a viewing DM creator to administer only their private alert', function () {
    $map = Map::factory()->create();
    $creator = User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Member])->for($map)))
        ->create();
    $directMessage = MapAlert::factory()->discordDm()->create([
        'map_id' => $map->id,
        'created_by_user_id' => $creator->id,
        'target_solarsystem_id' => makeSolarsystem(30009505),
    ]);
    $channel = MapAlert::factory()->discordChannel()->create([
        'map_id' => $map->id,
        'created_by_user_id' => $creator->id,
        'target_solarsystem_id' => makeSolarsystem(30009506),
    ]);

    expect($creator->can('update', $directMessage))->toBeTrue()
        ->and($creator->can('delete', $directMessage))->toBeTrue()
        ->and($creator->can('update', $channel))->toBeFalse()
        ->and($creator->can('delete', $channel))->toBeFalse();

    actingAs($creator)
        ->put(route('map-alerts.update', $directMessage), [])
        ->assertForbidden();
    actingAs($creator)
        ->put(route('map-alerts.state.update', $directMessage), ['enabled' => false])
        ->assertRedirect();
    actingAs($creator)
        ->delete(route('map-alerts.destroy', $channel))
        ->assertForbidden();
    actingAs($creator)
        ->delete(route('map-alerts.destroy', $directMessage))
        ->assertRedirect();

    expect(MapAlert::withTrashed()->findOrFail($directMessage->id)->trashed())->toBeTrue()
        ->and($channel->fresh())->not->toBeNull();
});

it('rejects invalid state changes without mutating the alert or history', function (mixed $enabled) {
    $map = Map::factory()->create();
    $manager = User::factory()->ownsMap($map)->create();
    $alert = MapAlert::factory()->discordChannel()->create([
        'map_id' => $map->id,
        'target_solarsystem_id' => makeSolarsystem(30009512),
    ]);

    actingAs($manager)
        ->put(route('map-alerts.state.update', $alert), ['enabled' => $enabled])
        ->assertInvalid(['enabled']);

    expect($alert->refresh()->is_active)->toBeTrue()
        ->and($alert->disabled_at)->toBeNull()
        ->and($alert->events()->count())->toBe(0);
})->with([
    'null' => null,
    'string' => 'disabled',
    'array' => [[]],
]);

it('does not let a former creator administer alerts they can no longer view', function () {
    $map = Map::factory()->create();
    $creator = User::factory()->create();
    $directMessage = MapAlert::factory()->discordDm()->create([
        'map_id' => $map->id,
        'created_by_user_id' => $creator->id,
        'target_solarsystem_id' => makeSolarsystem(30009507),
    ]);
    $webhook = MapAlert::factory()->webhook()->create([
        'map_id' => $map->id,
        'created_by_user_id' => $creator->id,
        'target_solarsystem_id' => makeSolarsystem(30009508),
    ]);

    expect($creator->can('update', $directMessage))->toBeFalse()
        ->and($creator->can('delete', $directMessage))->toBeFalse()
        ->and($creator->can('update', $webhook))->toBeFalse()
        ->and($creator->can('delete', $webhook))->toBeFalse()
        ->and($webhook->delivery_type)->toBe(MapAlertDeliveryType::Webhook);
});

it('keeps disabled bot alerts on account settings after Discord is unlinked', function () {
    $user = User::factory()->has(Character::factory())->create();
    $user->update(['preferred_character_id' => $user->characters->first()->id]);
    $user->refresh();
    $map = Map::factory()->create();
    $alert = MapAlert::factory()
        ->discordDm()
        ->disabled(MapAlertDisabledReason::DiscordAccountDisconnected)
        ->create([
            'map_id' => $map->id,
            'created_by_user_id' => $user->id,
            'target_solarsystem_id' => makeSolarsystem(30009504),
        ]);

    actingAs($user)
        ->get(route('settings.show'))
        ->assertSuccessful()
        ->assertInertia(fn (Assert $page) => $page
            ->where('discordAccount', null)
            ->has('discordAlerts', 1)
            ->where('discordAlerts.0.id', $alert->id)
            ->where('discordAlerts.0.is_active', false)
            ->where('discordAlerts.0.disabled_reason', 'discord_account_disconnected')
            ->etc());
});
