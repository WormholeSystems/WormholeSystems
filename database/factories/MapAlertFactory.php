<?php

declare(strict_types=1);

namespace Database\Factories;

use App\Enums\JumpShipType;
use App\Enums\MapAlertDeliveryType;
use App\Enums\MapAlertDisabledReason;
use App\Enums\MapAlertMentionMode;
use App\Enums\MapAlertType;
use App\Models\Map;
use App\Models\MapAlert;
use App\Models\MapWebhook;
use App\Models\Solarsystem;
use App\Models\User;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends Factory<MapAlert>
 */
final class MapAlertFactory extends Factory
{
    /**
     * Define the model's default state.
     *
     * @return array<string, mixed>
     */
    public function definition(): array
    {
        return [
            'map_webhook_id' => MapWebhook::factory(),
            'map_id' => fn (array $attributes): int => MapWebhook::query()->whereKey($attributes['map_webhook_id'])->value('map_id'),
            'created_by_user_id' => null,
            'delivery_type' => MapAlertDeliveryType::Webhook,
            'map_webhook_role_id' => null,
            'mention_mode' => MapAlertMentionMode::None,
            'discord_guild_id' => null,
            'discord_channel_id' => null,
            'discord_role_id' => null,
            'type' => MapAlertType::Proximity,
            'target_solarsystem_id' => fn () => Solarsystem::query()->inRandomOrder()->first()->id,
            'max_jumps' => fake()->numberBetween(1, 15),
            'is_active' => true,
        ];
    }

    public function inactive(): self
    {
        return $this->state(fn (): array => ['is_active' => false]);
    }

    public function webhook(): self
    {
        return $this->state(fn (): array => [
            'delivery_type' => MapAlertDeliveryType::Webhook,
            'map_webhook_id' => MapWebhook::factory(),
            'created_by_user_id' => null,
            'discord_guild_id' => null,
            'discord_channel_id' => null,
            'discord_role_id' => null,
        ]);
    }

    public function discordDm(): self
    {
        return $this->state(fn (): array => [
            'delivery_type' => MapAlertDeliveryType::DiscordDm,
            'map_webhook_id' => null,
            'map_id' => Map::factory(),
            'map_webhook_role_id' => null,
            'created_by_user_id' => User::factory(),
            'discord_guild_id' => null,
            'discord_channel_id' => null,
            'discord_role_id' => null,
        ]);
    }

    public function discordChannel(?string $guildId = null, ?string $channelId = null): self
    {
        return $this->state(fn (): array => [
            'delivery_type' => MapAlertDeliveryType::DiscordChannel,
            'map_webhook_id' => null,
            'map_id' => Map::factory(),
            'map_webhook_role_id' => null,
            'created_by_user_id' => User::factory(),
            'discord_guild_id' => $guildId ?? (string) fake()->numberBetween(100000000000000000, 999999999999999999),
            'discord_channel_id' => $channelId ?? (string) fake()->numberBetween(100000000000000000, 999999999999999999),
        ]);
    }

    public function mentionsCreator(): self
    {
        return $this->state(fn (): array => [
            'mention_mode' => MapAlertMentionMode::Creator,
            'discord_role_id' => null,
            'map_webhook_role_id' => null,
        ]);
    }

    public function mentionsRole(?string $discordRoleId = null): self
    {
        return $this->state(fn (): array => [
            'mention_mode' => MapAlertMentionMode::Role,
            'discord_role_id' => $discordRoleId ?? (string) fake()->numberBetween(100000000000000000, 999999999999999999),
        ]);
    }

    public function disabled(MapAlertDisabledReason $reason = MapAlertDisabledReason::Manual): self
    {
        return $this->state(fn (): array => [
            'is_active' => false,
            'disabled_at' => now(),
            'disabled_reason' => $reason,
        ]);
    }

    public function jumpRange(JumpShipType $ship = JumpShipType::Dreadnought, int $jdcLevel = 5, bool $includeHighsec = false): self
    {
        return $this->state(fn (): array => [
            'type' => MapAlertType::JumpRange,
            'ship_type' => $ship,
            'jdc_level' => $jdcLevel,
            'include_highsec' => $includeHighsec,
            'max_jumps' => null,
        ]);
    }

    /**
     * @param  array<int, array{subject: string, side: string, mode: string, ids: int[]}>  $filters
     */
    public function killmail(array $filters = []): self
    {
        return $this->state(fn (): array => [
            'type' => MapAlertType::Killmail,
            'target_solarsystem_id' => null,
            'filters' => $filters,
        ]);
    }
}
