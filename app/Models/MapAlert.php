<?php

declare(strict_types=1);

namespace App\Models;

use App\Casts\KillmailFiltersCast;
use App\Enums\JumpShipType;
use App\Enums\KillmailFilterMatch;
use App\Enums\MapAlertDeliveryType;
use App\Enums\MapAlertDisabledReason;
use App\Enums\MapAlertMentionMode;
use App\Enums\MapAlertType;
use App\Services\Killmails\KillmailFilterRule;
use Carbon\CarbonImmutable;
use Database\Factories\MapAlertFactory;
use Illuminate\Database\Eloquent\Attributes\UseFactory;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Illuminate\Database\Eloquent\SoftDeletes;
use Illuminate\Support\Collection;

/**
 * An alert that delivers to a webhook or Discord when a target system comes
 * within range of the map (proximity), a matching killmail occurs within range
 * (killmail), or a new k-space exit lands within a capital's jump range of the
 * target system (jump_range).
 *
 * @property int $id
 * @property int $map_id
 * @property int|null $created_by_user_id
 * @property MapAlertDeliveryType $delivery_type
 * @property int|null $map_webhook_id
 * @property int|null $map_webhook_role_id
 * @property MapAlertMentionMode $mention_mode
 * @property string|null $discord_guild_id
 * @property string|null $discord_channel_id
 * @property string|null $discord_role_id
 * @property MapAlertType $type
 * @property int|null $target_solarsystem_id
 * @property int|null $origin_solarsystem_id
 * @property JumpShipType|null $ship_type
 * @property int|null $jdc_level
 * @property bool $include_highsec
 * @property int|null $max_jumps
 * @property Collection<int, KillmailFilterRule> $filters
 * @property KillmailFilterMatch $filter_match
 * @property bool $is_active
 * @property CarbonImmutable|null $last_fired_at
 * @property CarbonImmutable|null $disabled_at
 * @property int|null $disabled_by_user_id
 * @property MapAlertDisabledReason|null $disabled_reason
 * @property CarbonImmutable|null $deleted_at
 * @property int|null $deleted_by_user_id
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Map $map
 * @property-read User|null $creator
 * @property-read User|null $disabledBy
 * @property-read User|null $deletedBy
 * @property-read MapWebhook|null $webhook
 * @property-read MapWebhookRole|null $role
 * @property-read Solarsystem|null $targetSolarsystem
 * @property-read Solarsystem|null $originSolarsystem
 * @property-read Collection<int, MapAlertDelivery> $deliveries
 * @property-read Collection<int, MapAlertEvent> $events
 */
#[UseFactory(MapAlertFactory::class)]
final class MapAlert extends Model
{
    /** @use HasFactory<MapAlertFactory> */
    use HasFactory, SoftDeletes;

    /** @param Builder<MapAlert> $query */
    public function scopeBot(Builder $query): void
    {
        $query->whereIn('delivery_type', MapAlertDeliveryType::botTypes());
    }

    /** @param Builder<MapAlert> $query */
    public function scopeWebhook(Builder $query): void
    {
        $query->where('delivery_type', MapAlertDeliveryType::Webhook);
    }

    /** @param Builder<MapAlert> $query */
    public function scopePrivate(Builder $query): void
    {
        $query->where('delivery_type', MapAlertDeliveryType::DiscordDm);
    }

    /** @param Builder<MapAlert> $query */
    public function scopeShared(Builder $query): void
    {
        $query->whereIn('delivery_type', MapAlertDeliveryType::sharedTypes());
    }

    /**
     * Whether delivering this alert depends on the creator having a linked Discord account.
     */
    public function requiresCreatorDiscordAccount(): bool
    {
        return $this->delivery_type === MapAlertDeliveryType::DiscordDm
            || ($this->delivery_type === MapAlertDeliveryType::DiscordChannel && $this->mention_mode === MapAlertMentionMode::Creator);
    }

    /**
     * The map this alert belongs to.
     *
     * @return BelongsTo<Map, $this>
     */
    public function map(): BelongsTo
    {
        return $this->belongsTo(Map::class);
    }

    /** @return BelongsTo<User, $this> */
    public function creator(): BelongsTo
    {
        return $this->belongsTo(User::class, 'created_by_user_id');
    }

    /** @return BelongsTo<User, $this> */
    public function disabledBy(): BelongsTo
    {
        return $this->belongsTo(User::class, 'disabled_by_user_id');
    }

    /** @return BelongsTo<User, $this> */
    public function deletedBy(): BelongsTo
    {
        return $this->belongsTo(User::class, 'deleted_by_user_id');
    }

    /**
     * The webhook destination this alert delivers to.
     *
     * @return BelongsTo<MapWebhook, $this>
     */
    public function webhook(): BelongsTo
    {
        return $this->belongsTo(MapWebhook::class, 'map_webhook_id');
    }

    /**
     * The role this alert pings, if any.
     *
     * @return BelongsTo<MapWebhookRole, $this>
     */
    public function role(): BelongsTo
    {
        return $this->belongsTo(MapWebhookRole::class, 'map_webhook_role_id');
    }

    /**
     * The system whose proximity to the map triggers this alert.
     *
     * @return BelongsTo<Solarsystem, $this>
     */
    public function targetSolarsystem(): BelongsTo
    {
        return $this->belongsTo(Solarsystem::class, 'target_solarsystem_id');
    }

    /**
     * The optional fixed starting point proximity is measured from; without one the
     * alert measures from any point of the chain.
     *
     * @return BelongsTo<Solarsystem, $this>
     */
    public function originSolarsystem(): BelongsTo
    {
        return $this->belongsTo(Solarsystem::class, 'origin_solarsystem_id');
    }

    /** @return HasMany<MapAlertDelivery, $this> */
    public function deliveries(): HasMany
    {
        return $this->hasMany(MapAlertDelivery::class);
    }

    /** @return HasMany<MapAlertEvent, $this> */
    public function events(): HasMany
    {
        return $this->hasMany(MapAlertEvent::class);
    }

    /**
     * @return array<string, string>
     */
    protected function casts(): array
    {
        return [
            'delivery_type' => MapAlertDeliveryType::class,
            'mention_mode' => MapAlertMentionMode::class,
            'type' => MapAlertType::class,
            'ship_type' => JumpShipType::class,
            'jdc_level' => 'integer',
            'include_highsec' => 'boolean',
            'max_jumps' => 'integer',
            'filters' => KillmailFiltersCast::class,
            'filter_match' => KillmailFilterMatch::class,
            'is_active' => 'boolean',
            'last_fired_at' => 'immutable_datetime',
            'disabled_at' => 'immutable_datetime',
            'disabled_reason' => MapAlertDisabledReason::class,
            'deleted_at' => 'immutable_datetime',
        ];
    }
}
