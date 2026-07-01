<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Database\Factories\MapWebhookRoleFactory;
use Illuminate\Database\Eloquent\Attributes\UseFactory;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;

/**
 * A reusable Discord role that an alert can ping when it fires.
 *
 * @property int $id
 * @property int $map_id
 * @property string $name
 * @property string $discord_role_id
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Map $map
 * @property-read Collection<int, MapAlert> $alerts
 */
#[UseFactory(MapWebhookRoleFactory::class)]
final class MapWebhookRole extends Model
{
    /** @use HasFactory<MapWebhookRoleFactory> */
    use HasFactory;

    /**
     * The map this role belongs to.
     *
     * @return BelongsTo<Map, $this>
     */
    public function map(): BelongsTo
    {
        return $this->belongsTo(Map::class);
    }

    /**
     * The alerts that ping this role.
     *
     * @return HasMany<MapAlert, $this>
     */
    public function alerts(): HasMany
    {
        return $this->hasMany(MapAlert::class);
    }
}
