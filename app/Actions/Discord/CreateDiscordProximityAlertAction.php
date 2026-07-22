<?php

declare(strict_types=1);

namespace App\Actions\Discord;

use App\Enums\MapAlertDeliveryType;
use App\Enums\MapAlertMentionMode;
use App\Enums\MapAlertType;
use App\Models\DiscordAccount;
use App\Models\Map;
use App\Models\MapAlert;
use App\Models\Solarsystem;
use App\Services\MapAlerts\MapAlertLifecycle;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Gate;

final readonly class CreateDiscordProximityAlertAction
{
    public function __construct(private MapAlertLifecycle $lifecycle) {}

    public function handle(
        DiscordAccount $account,
        int $mapId,
        int $solarsystemId,
        int $jumps,
        ?MapAlertDeliveryType $delivery,
        ?MapAlertMentionMode $mentionMode,
        ?string $guildId,
        ?string $channelId,
        ?string $roleId,
    ): string {
        $map = Map::query()->find($mapId);
        $target = Solarsystem::query()->find($solarsystemId);
        if ($map === null || $target === null || ! in_array($delivery, [MapAlertDeliveryType::DiscordDm, MapAlertDeliveryType::DiscordChannel], true) || ! $mentionMode instanceof MapAlertMentionMode || $jumps < 1 || $jumps > 20) {
            return 'Invalid map, system, destination, or jump count.';
        }

        $isChannel = $delivery === MapAlertDeliveryType::DiscordChannel;

        if (Gate::forUser($account->user)->denies($isChannel ? 'manageAccess' : 'view', $map)) {
            return $isChannel
                ? 'Only map managers can create channel alerts.'
                : 'You no longer have access to that map.';
        }

        if ($isChannel && (blank($guildId) || blank($channelId))) {
            return 'Channel alerts must be created inside a server channel.';
        }

        if ($delivery === MapAlertDeliveryType::DiscordDm && ($mentionMode !== MapAlertMentionMode::None || filled($roleId))) {
            return 'Mention options can only be used for channel alerts.';
        }

        if (($mentionMode === MapAlertMentionMode::Role) !== filled($roleId)) {
            return 'Select a role only when the mention option is A role.';
        }

        DB::transaction(function () use ($account, $map, $target, $jumps, $delivery, $mentionMode, $guildId, $channelId, $roleId, $isChannel): void {
            $alert = MapAlert::query()->create([
                'map_id' => $map->id,
                'created_by_user_id' => $account->user_id,
                'delivery_type' => $delivery,
                'map_webhook_id' => null,
                'map_webhook_role_id' => null,
                'mention_mode' => $isChannel ? $mentionMode : MapAlertMentionMode::None,
                'discord_guild_id' => $isChannel ? $guildId : null,
                'discord_channel_id' => $isChannel ? $channelId : null,
                'discord_role_id' => $isChannel ? $roleId : null,
                'type' => MapAlertType::Proximity,
                'target_solarsystem_id' => $target->id,
                'max_jumps' => $jumps,
                'is_active' => true,
            ]);
            $this->lifecycle->created($alert, $account->user);
        });

        return sprintf('Alert created for **%s** within %d jumps of **%s**.', $target->name, $jumps, $map->name);
    }
}
