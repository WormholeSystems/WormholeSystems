<?php

declare(strict_types=1);

use App\Enums\MapAlertDeliveryType;
use App\Enums\MapAlertDisabledReason;
use App\Enums\MapAlertEventAction;
use App\Enums\MapAlertMentionMode;
use App\Enums\MapAlertType;
use App\Models\DiscordAccount;
use App\Models\MapAlert;
use App\Models\MapAlertDelivery;
use App\Models\MapAlertEvent;
use App\Models\MapSolarsystem;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\Schema;

uses(RefreshDatabase::class);

beforeEach(function (): void {
    makeSolarsystem(30000142);
});

it('migrates the unified alert schema', function (): void {
    expect(Schema::hasColumns('map_alerts', [
        'created_by_user_id',
        'delivery_type',
        'mention_mode',
        'discord_guild_id',
        'discord_channel_id',
        'discord_role_id',
        'disabled_at',
        'disabled_by_user_id',
        'disabled_reason',
        'deleted_at',
        'deleted_by_user_id',
    ]))->toBeTrue()
        ->and(Schema::hasColumns('map_alert_events', [
            'map_alert_id',
            'map_id',
            'actor_user_id',
            'actor_name',
            'action',
            'snapshot',
            'reason',
            'created_at',
        ]))->toBeTrue()
        ->and(Schema::hasTable('map_alert_deliveries'))->toBeTrue()
        ->and(Schema::hasTable('discord_alert_deliveries'))->toBeFalse()
        ->and(Schema::hasTable('discord_proximity_subscriptions'))->toBeFalse();
});

it('casts destinations and filters alert scopes', function (): void {
    $webhook = MapAlert::factory()->webhook()->create();
    $directMessage = MapAlert::factory()->discordDm()->mentionsCreator()->create();
    $channel = MapAlert::factory()->discordChannel('111111111111111111', '222222222222222222')->mentionsRole('333333333333333333')->create();

    expect($webhook->delivery_type)->toBe(MapAlertDeliveryType::Webhook)
        ->and($webhook->type)->toBe(MapAlertType::Proximity)
        ->and(MapAlertType::Proximity->value)->toBe('proximity')
        ->and(MapAlertType::Killmail->value)->toBe('killmail')
        ->and(MapAlertType::JumpRange->value)->toBe('jump_range')
        ->and($directMessage->delivery_type)->toBe(MapAlertDeliveryType::DiscordDm)
        ->and($directMessage->mention_mode)->toBe(MapAlertMentionMode::Creator)
        ->and($channel->delivery_type)->toBe(MapAlertDeliveryType::DiscordChannel)
        ->and($channel->discord_guild_id)->toBe('111111111111111111')
        ->and(MapAlert::query()->bot()->pluck('id')->all())->toEqualCanonicalizing([$directMessage->id, $channel->id])
        ->and(MapAlert::query()->webhook()->pluck('id')->all())->toBe([$webhook->id])
        ->and(MapAlert::query()->private()->pluck('id')->all())->toBe([$directMessage->id])
        ->and(MapAlert::query()->shared()->pluck('id')->all())->toEqualCanonicalizing([$webhook->id, $channel->id]);
});

it('relates alerts through their creator and preserves delivery reservations', function (): void {
    $alert = MapAlert::factory()->discordDm()->create();
    $account = DiscordAccount::factory()->for($alert->creator)->create();
    $placement = MapSolarsystem::factory()->for($alert->map)->create([
        'solarsystem_id' => 30000142,
    ]);
    $delivery = MapAlertDelivery::query()->create([
        'map_alert_id' => $alert->id,
        'map_solarsystem_id' => $placement->id,
    ]);

    expect($alert->creator->createdMapAlerts)->toHaveCount(1)
        ->and($account->user->createdMapAlerts)->toHaveCount(1)
        ->and($delivery->alert->is($alert))->toBeTrue()
        ->and($alert->deliveries)->toHaveCount(1);
});

it('casts disabled state and prevents event mutation', function (): void {
    $alert = MapAlert::factory()->discordDm()->disabled(MapAlertDisabledReason::DeliveryFailed)->create();
    $event = MapAlertEvent::factory()->for($alert, 'alert')->create([
        'map_id' => $alert->map_id,
        'snapshot' => ['delivery_type' => $alert->delivery_type->value],
    ]);

    expect($alert->disabled_reason)->toBe(MapAlertDisabledReason::DeliveryFailed)
        ->and($alert->disabled_at)->not->toBeNull()
        ->and($event->action)->toBe(MapAlertEventAction::Created)
        ->and($event->snapshot)->toBe(['delivery_type' => 'discord_dm']);

    expect(fn () => $event->update(['reason' => 'changed']))
        ->toThrow(LogicException::class, 'Map alert events are immutable.');
});
