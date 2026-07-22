<?php

declare(strict_types=1);

namespace App\Actions\Discord;

use App\Enums\MapAlertDeliveryType;
use App\Enums\MapAlertType;
use App\Models\MapAlert;
use App\Models\Solarsystem;

final readonly class DiscordAlertFormatter
{
    public function format(MapAlert $alert, bool $showCreator = false): string
    {
        $destination = match ($alert->delivery_type) {
            MapAlertDeliveryType::Webhook => 'webhook',
            MapAlertDeliveryType::DiscordDm => 'DM',
            MapAlertDeliveryType::DiscordChannel => $alert->discord_channel_id === null ? 'channel' : '<#'.$alert->discord_channel_id.'>',
        };
        $creator = $showCreator && $alert->creator !== null ? ' by '.$alert->creator->name : '';

        return sprintf('`%d` %s: %s via %s%s%s', $alert->id, $alert->map->name, $this->description($alert), $destination, $creator, $alert->is_active ? '' : ' (disabled)');
    }

    private function description(MapAlert $alert): string
    {
        return match ($alert->type) {
            MapAlertType::Killmail => sprintf('killmails within %d jumps', $alert->max_jumps ?? 0),
            MapAlertType::JumpRange => sprintf('capital range of %s', $alert->targetSolarsystem->name),
            MapAlertType::Proximity => $alert->originSolarsystem instanceof Solarsystem
                ? sprintf('%s within %d jumps of %s', $alert->targetSolarsystem->name, $alert->max_jumps ?? 0, $alert->originSolarsystem->name)
                : sprintf('%s within %d jumps', $alert->targetSolarsystem->name, $alert->max_jumps ?? 0),
        };
    }
}
