<?php

declare(strict_types=1);

namespace App\Services\Discord;

use App\Enums\MapAlertDeliveryType;
use App\Enums\MapAlertMentionMode;
use App\Models\MapAlert;
use Illuminate\Support\Facades\Cache;
use RuntimeException;

final readonly class DiscordBotDelivery
{
    public function __construct(private DiscordHttp $http) {}

    /** @param array<string, mixed> $embed */
    public function deliver(MapAlert $alert, array $embed, string $nonce): void
    {
        $channelId = $alert->discord_channel_id;
        $content = null;
        $allowedMentions = [
            'parse' => [],
            'users' => [],
            'roles' => [],
        ];

        if ($alert->delivery_type === MapAlertDeliveryType::DiscordDm) {
            $discordUserId = $alert->creator?->discordAccount?->discord_user_id;
            if (! is_string($discordUserId) || $discordUserId === '') {
                throw new RuntimeException('Discord direct message alert does not have a recipient.');
            }

            $channelId = $this->directMessageChannelId($discordUserId);
        } elseif ($alert->mention_mode === MapAlertMentionMode::Creator) {
            $discordUserId = $alert->creator?->discordAccount?->discord_user_id;
            if (! is_string($discordUserId) || $discordUserId === '') {
                throw new RuntimeException('Discord creator mention does not have an identity.');
            }

            $content = '<@'.$discordUserId.'>';
            $allowedMentions['users'] = [$discordUserId];
        } elseif ($alert->mention_mode === MapAlertMentionMode::Everyone) {
            $content = '@everyone';
            $allowedMentions['parse'] = ['everyone'];
        } elseif ($alert->mention_mode === MapAlertMentionMode::Role) {
            if (! is_string($alert->discord_role_id) || $alert->discord_role_id === '') {
                throw new RuntimeException('Discord role mention does not have a role.');
            }

            if ($alert->discord_role_id === $alert->discord_guild_id) {
                // Discord's @everyone role shares the guild's id; it only pings through the
                // special "everyone" mention, never through a <@&id> role mention.
                $content = '@everyone';
                $allowedMentions['parse'] = ['everyone'];
            } else {
                $content = '<@&'.$alert->discord_role_id.'>';
                $allowedMentions['roles'] = [$alert->discord_role_id];
            }
        }

        if (! is_string($channelId) || $channelId === '') {
            throw new RuntimeException('Discord did not return a destination channel.');
        }

        $this->http->bot()->post('/channels/'.$channelId.'/messages', [
            'content' => $content,
            'embeds' => [$embed],
            'allowed_mentions' => $allowedMentions,
            'nonce' => $nonce,
            'enforce_nonce' => true,
        ])->throw();
    }

    /**
     * DM channel ids are stable per Discord user, so open the channel once and cache it.
     */
    private function directMessageChannelId(string $discordUserId): ?string
    {
        return Cache::rememberForever(
            'discord:dm-channel:'.$discordUserId,
            fn (): ?string => $this->http->bot()->post('/users/@me/channels', [
                'recipient_id' => $discordUserId,
            ])->throw()->json('id'),
        );
    }
}
