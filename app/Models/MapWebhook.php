<?php

declare(strict_types=1);

namespace App\Models;

use App\Enums\MapWebhookType;
use Carbon\CarbonImmutable;
use Database\Factories\MapWebhookFactory;
use Illuminate\Database\Eloquent\Attributes\UseFactory;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * A Discord webhook that fires when a target system comes within range of the map.
 *
 * @property int $id
 * @property int $map_id
 * @property string $name
 * @property string $discord_webhook_url
 * @property MapWebhookType $type
 * @property int $target_solarsystem_id
 * @property int $max_jumps
 * @property bool $is_active
 * @property CarbonImmutable|null $last_fired_at
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Map $map
 * @property-read Solarsystem $targetSolarsystem
 */
#[UseFactory(MapWebhookFactory::class)]
final class MapWebhook extends Model
{
    /** @use HasFactory<MapWebhookFactory> */
    use HasFactory;

    /**
     * The map this webhook belongs to.
     *
     * @return BelongsTo<Map, $this>
     */
    public function map(): BelongsTo
    {
        return $this->belongsTo(Map::class);
    }

    /**
     * The system whose proximity to the map triggers this webhook.
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
            'discord_webhook_url' => 'encrypted',
            'max_jumps' => 'integer',
            'is_active' => 'boolean',
            'last_fired_at' => 'immutable_datetime',
        ];
    }
}
