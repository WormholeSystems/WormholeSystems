<?php

declare(strict_types=1);

namespace App\Actions\Discord;

use App\Enums\JumpShipType;
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

final readonly class CreateDiscordAlertAction
{
    public function __construct(private MapAlertLifecycle $lifecycle) {}

    public function handle(
        DiscordAccount $account,
        MapAlertType $type,
        ?MapAlertDeliveryType $delivery,
        int $mapId,
        ?int $solarsystemId,
        ?int $jumps,
        ?JumpShipType $shipType,
        ?int $jdcLevel,
        bool $includeHighsec,
        ?MapAlertMentionMode $mentionMode,
        ?string $guildId,
        ?string $channelId,
        ?string $roleId,
    ): string {
        $map = Map::query()->find($mapId);
        if ($map === null || ! in_array($delivery, [MapAlertDeliveryType::DiscordDm, MapAlertDeliveryType::DiscordChannel], true) || ! $mentionMode instanceof MapAlertMentionMode) {
            return 'Invalid map or destination.';
        }

        $needsTarget = $type !== MapAlertType::Killmail;
        $target = $needsTarget && $solarsystemId !== null ? Solarsystem::query()->find($solarsystemId) : null;
        if ($needsTarget && $target === null) {
            return 'That target system is unavailable.';
        }

        $needsJumps = $type !== MapAlertType::JumpRange;
        if ($needsJumps && ($jumps === null || $jumps < 1 || $jumps > 20)) {
            return 'The jump count must be between 1 and 20.';
        }

        if ($type === MapAlertType::JumpRange && (! $shipType instanceof JumpShipType || $jdcLevel === null || $jdcLevel < 1 || $jdcLevel > 5)) {
            return 'Jump-range alerts need a ship class and a calibration level between 1 and 5.';
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

        if (! $isChannel && ($mentionMode !== MapAlertMentionMode::None || filled($roleId))) {
            return 'Mention options can only be used for channel alerts.';
        }

        if (($mentionMode === MapAlertMentionMode::Role) !== filled($roleId)) {
            return 'Select a role only when the mention option is A role.';
        }

        DB::transaction(function () use ($account, $map, $type, $target, $jumps, $shipType, $jdcLevel, $includeHighsec, $delivery, $mentionMode, $guildId, $channelId, $roleId, $isChannel): void {
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
                'type' => $type,
                'target_solarsystem_id' => $target?->id,
                'max_jumps' => $type === MapAlertType::JumpRange ? null : $jumps,
                'ship_type' => $type === MapAlertType::JumpRange ? $shipType : null,
                'jdc_level' => $type === MapAlertType::JumpRange ? $jdcLevel : null,
                'include_highsec' => $type === MapAlertType::JumpRange && $includeHighsec,
                'is_active' => true,
            ]);
            $this->lifecycle->created($alert, $account->user);
        });

        return match ($type) {
            MapAlertType::Proximity => sprintf('Alert created for **%s** within %d jumps of **%s**.', $target->name, $jumps, $map->name),
            MapAlertType::JumpRange => sprintf('Alert created for exits within %.1f ly of **%s** on **%s**.', $shipType->maxRangeLy($jdcLevel), $target->name, $map->name),
            MapAlertType::Killmail => sprintf('Alert created for kills within %d jumps of the **%s** chain.', $jumps, $map->name),
        };
    }
}
