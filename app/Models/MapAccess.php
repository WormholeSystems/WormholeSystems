<?php

declare(strict_types=1);

namespace App\Models;

use App\Enums\Permission;
use Carbon\CarbonImmutable;
use Database\Factories\MapAccessFactory;
use Illuminate\Database\Eloquent\Attributes\UseFactory;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\MorphTo;

/**
 * Represents an access control entry for a map.
 *
 * @property int $id
 * @property string $accessible_type
 * @property int $accessible_id
 * @property int $map_id
 * @property Permission $permission
 * @property bool $is_owner
 * @property CarbonImmutable|null $expires_at
 * @property-read Alliance|Corporation|Character $accessible
 * @property-read Map $map
 */
#[UseFactory(MapAccessFactory::class)]
final class MapAccess extends Model
{
    /** @use HasFactory<MapAccessFactory> */
    use HasFactory;

    protected $table = 'map_access';

    public function accessible(): MorphTo
    {
        return $this->morphTo();
    }

    public function map(): BelongsTo
    {
        return $this->belongsTo(Map::class);
    }

    /**
     * Scope to only include non-expired access entries.
     *
     * @param  Builder<self>  $query
     * @return Builder<self>
     */
    public function scopeNotExpired(Builder $query): Builder
    {
        return $query->where(fn (Builder $query) => $query
            ->whereNull('expires_at')
            ->orWhere('expires_at', '>', now())
        );
    }

    public function isExpired(): bool
    {
        return $this->expires_at !== null && $this->expires_at->isPast();
    }

    protected function casts(): array
    {
        return [
            'permission' => Permission::class,
            'expires_at' => 'immutable_datetime',
        ];
    }
}
