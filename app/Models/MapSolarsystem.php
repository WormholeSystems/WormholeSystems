<?php

declare(strict_types=1);

namespace App\Models;

use App\Builders\MapSolarsystemBuilder;
use App\Enums\MapSolarsystemStatus;
use Database\Factories\MapSolarsystemFactory;
use Illuminate\Database\Eloquent\Attributes\UseEloquentBuilder;
use Illuminate\Database\Eloquent\Attributes\UseFactory;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Illuminate\Database\Eloquent\Relations\HasOne;
use OwenIt\Auditing\Auditable;
use OwenIt\Auditing\Models\Audit;
use Staudenmeir\EloquentHasManyDeep\HasManyDeep;
use Staudenmeir\EloquentHasManyDeep\HasRelationships;

/**
 * Represents a solarsystem on a map
 *
 * @property int $id
 * @property int $map_id
 * @property int $solarsystem_id
 * @property string|null $alias
 * @property string|null $occupier_alias
 * @property float|null $position_x
 * @property float|null $position_y
 * @property MapSolarsystemStatus $status
 * @property bool $pinned
 * @property-read int|null $signatures_count
 * @property-read Solarsystem $solarsystem
 * @property-read Map $map
 * @property-read Collection<int,MapConnection> $connectionsTo
 * @property-read Collection<int,MapConnection> $connectionsFrom
 * @property-read WormholeSystem|null $wormholeSystem
 * @property-read Collection<int,Signature> $signatures
 * @property-read Collection<int,Audit> $audits
 * @property-read Collection<int,Wormhole> $wormholes
 */
#[UseFactory(MapSolarsystemFactory::class)]
#[UseEloquentBuilder(MapSolarsystemBuilder::class)]
final class MapSolarsystem extends Model implements \OwenIt\Auditing\Contracts\Auditable
{
    use Auditable;

    /** @use HasFactory<MapSolarsystemFactory> */
    use HasFactory;

    use HasRelationships;

    /**
     * @var Collection<int,MapConnection>|null
     */
    public ?Collection $connections {
        get {
            $mergedConnections = $this->connectionsTo->merge($this->connectionsFrom);

            $this->setRelation('connections', $mergedConnections);

            return $mergedConnections;
        }
    }

    /**
     * @return BelongsTo<Solarsystem,$this>
     */
    public function solarsystem(): BelongsTo
    {
        return $this->belongsTo(Solarsystem::class);
    }

    /**
     * @return BelongsTo<Map,$this>
     */
    public function map(): BelongsTo
    {
        return $this->belongsTo(Map::class);
    }

    /**
     * @return HasMany<MapConnection,$this>
     */
    public function connectionsTo(): HasMany
    {
        return $this->hasMany(MapConnection::class, 'from_map_solarsystem_id');
    }

    /**
     * @return HasMany<MapConnection,$this>
     */
    public function connectionsFrom(): HasMany
    {
        return $this->hasMany(MapConnection::class, 'to_map_solarsystem_id');
    }

    /**
     * @return HasOne<WormholeSystem,$this>
     */
    public function wormholeSystem(): HasOne
    {
        return $this->hasOne(WormholeSystem::class, 'id', 'solarsystem_id');
    }

    /**
     * @return HasMany<Signature,$this>
     */
    public function signatures(): HasMany
    {
        return $this->hasMany(Signature::class, 'map_solarsystem_id');
    }

    public function wormholes(): HasManyDeep
    {
        return $this->hasManyDeepFromRelations(
            $this->wormholeSystem(),
            new WormholeSystem()->wormholeStatics(),
            new WormholeStatic()->wormhole(),
        );
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
