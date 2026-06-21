<?php

declare(strict_types=1);

namespace App\Models;

use App\Enums\MapSolarsystemStatus;
use Database\Factories\MapSolarsystemDetailsFactory;
use Illuminate\Database\Eloquent\Attributes\UseFactory;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasOne;
use OwenIt\Auditing\Auditable;
use OwenIt\Auditing\Models\Audit;

/**
 * Persistent per-map intel about a solarsystem that survives the system being removed from
 * (and re-added to) the map. The canvas placement lives on {@see MapSolarsystem}.
 *
 * @property int $id
 * @property int $map_id
 * @property int $solarsystem_id
 * @property string|null $occupier_alias
 * @property MapSolarsystemStatus $status
 * @property string|null $notes
 * @property-read Solarsystem $solarsystem
 * @property-read Map $map
 * @property-read MapSolarsystem|null $mapSolarsystem
 * @property-read Collection<int,Audit> $audits
 */
#[UseFactory(MapSolarsystemDetailsFactory::class)]
final class MapSolarsystemDetails extends Model implements \OwenIt\Auditing\Contracts\Auditable
{
    use Auditable;

    /** @use HasFactory<MapSolarsystemDetailsFactory> */
    use HasFactory;

    /**
     * @return BelongsTo<Map,$this>
     */
    public function map(): BelongsTo
    {
        return $this->belongsTo(Map::class);
    }

    /**
     * @return BelongsTo<Solarsystem,$this>
     */
    public function solarsystem(): BelongsTo
    {
        return $this->belongsTo(Solarsystem::class);
    }

    /**
     * The current placement of this system on the map, if any.
     *
     * @return HasOne<MapSolarsystem,$this>
     */
    public function mapSolarsystem(): HasOne
    {
        return $this->hasOne(MapSolarsystem::class);
    }

    public function hideNotes(bool $hide = true): self
    {
        $this->makeHiddenIf($hide, 'notes');

        return $this;
    }

    protected function casts(): array
    {
        return [
            'status' => MapSolarsystemStatus::class,
            'created_at' => 'immutable_datetime',
            'updated_at' => 'immutable_datetime',
        ];
    }
}
