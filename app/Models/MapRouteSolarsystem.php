<?php

declare(strict_types=1);

namespace App\Models;

use Database\Factories\MapRouteSolarsystemFactory;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * A solarsystem which distance should be tracked on a map. Shared with every map user when
 * user_id is null, personal to that user otherwise.
 *
 * @property int $id
 * @property int $map_id
 * @property int|null $user_id
 * @property int $solarsystem_id
 * @property bool $is_pinned
 * @property-read Solarsystem $solarsystem
 * @property-read Map $map
 * @property-read User|null $user
 */
final class MapRouteSolarsystem extends Model
{
    /** @use HasFactory<MapRouteSolarsystemFactory> */
    use HasFactory;

    /**
     * The related solarsystem.
     *
     * @return BelongsTo<Solarsystem,$this>
     */
    public function solarsystem(): BelongsTo
    {
        return $this->belongsTo(Solarsystem::class);
    }

    /**
     * The related map.
     *
     * @return BelongsTo<Map,$this>
     */
    public function map(): BelongsTo
    {
        return $this->belongsTo(Map::class);
    }

    /**
     * The owner of a personal row, null for shared rows.
     *
     * @return BelongsTo<User,$this>
     */
    public function user(): BelongsTo
    {
        return $this->belongsTo(User::class);
    }

    public function isShared(): bool
    {
        return $this->user_id === null;
    }

    /** @param Builder<MapRouteSolarsystem> $query */
    public function scopeShared(Builder $query): void
    {
        $query->whereNull('user_id');
    }

    /**
     * Shared rows plus, when a user is given, that user's personal rows.
     *
     * @param  Builder<MapRouteSolarsystem>  $query
     */
    public function scopeVisibleTo(Builder $query, ?User $user): void
    {
        $query->where(function (Builder $query) use ($user): void {
            $query->shared();

            if ($user instanceof User) {
                $query->orWhere('user_id', $user->id);
            }
        });
    }

    protected function casts(): array
    {
        return [
            'is_pinned' => 'boolean',
        ];
    }
}
