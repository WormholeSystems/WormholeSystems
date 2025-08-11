<?php

declare(strict_types=1);

namespace App\Models;

use Database\Factories\MapRouteSolarsystemFactory;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * A solarsystem which distance should be tracked on a map
 *
 * @property int $id
 * @property int $map_id
 * @property int $solarsystem_id
 * @property bool $is_pinned
 * @property-read Solarsystem $solarsystem
 * @property-read Map $map
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

    protected function casts(): array
    {
        return [
            'is_pinned' => 'boolean',
        ];
    }
}
