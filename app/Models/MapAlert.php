<?php

declare(strict_types=1);

namespace App\Models;

use App\Casts\KillmailFiltersCast;
use App\Enums\JumpShipType;
use App\Enums\KillmailFilterMatch;
use App\Enums\MapWebhookType;
use App\Services\Killmails\KillmailFilterRule;
use Carbon\CarbonImmutable;
use Database\Factories\MapAlertFactory;
use Illuminate\Database\Eloquent\Attributes\UseFactory;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Support\Collection;

/**
 * An alert that fires to a webhook (and optionally pings a role) when a target
 * system comes within range of the map (proximity), a matching killmail occurs
 * within range (killmail), or a new k-space exit lands within a capital's jump
 * range of the target system (jump_range).
 *
 * @property int $id
 * @property int $map_id
 * @property int $map_webhook_id
 * @property int|null $map_webhook_role_id
 * @property MapWebhookType $type
 * @property int|null $target_solarsystem_id
 * @property JumpShipType|null $ship_type
 * @property int|null $jdc_level
 * @property bool $include_highsec
 * @property int|null $max_jumps
 * @property Collection<int, KillmailFilterRule> $filters
 * @property KillmailFilterMatch $filter_match
 * @property bool $is_active
 * @property CarbonImmutable|null $last_fired_at
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Map $map
 * @property-read MapWebhook $webhook
 * @property-read MapWebhookRole|null $role
 * @property-read Solarsystem|null $targetSolarsystem
 */
#[UseFactory(MapAlertFactory::class)]
final class MapAlert extends Model
{
    /** @use HasFactory<MapAlertFactory> */
    use HasFactory;

    /**
     * The map this alert belongs to.
     *
     * @return BelongsTo<Map, $this>
     */
    public function map(): BelongsTo
    {
        return $this->belongsTo(Map::class);
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
     * @return array<string, string>
     */
    protected function casts(): array
    {
        return [
            'type' => MapWebhookType::class,
            'ship_type' => JumpShipType::class,
            'jdc_level' => 'integer',
            'include_highsec' => 'boolean',
            'max_jumps' => 'integer',
            'filters' => KillmailFiltersCast::class,
            'filter_match' => KillmailFilterMatch::class,
            'is_active' => 'boolean',
            'last_fired_at' => 'immutable_datetime',
        ];
    }
}
