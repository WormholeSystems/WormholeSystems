<?php

declare(strict_types=1);

namespace App\Models;

use App\Builders\MapSolarsystemBuilder;
use App\Collections\MapSolarsystemCollection;
use App\Relations\HasManyMapConnections;
use Database\Factories\MapSolarsystemFactory;
use Illuminate\Database\Eloquent\Attributes\CollectedBy;
use Illuminate\Database\Eloquent\Attributes\UseEloquentBuilder;
use Illuminate\Database\Eloquent\Attributes\UseFactory;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Illuminate\Database\Eloquent\Relations\HasOne;
use Staudenmeir\EloquentHasManyDeep\HasManyDeep;
use Staudenmeir\EloquentHasManyDeep\HasRelationships;

/**
 * Represents the placement of a solarsystem on a map (canvas position, alias, pinned state).
 * Persistent per-map intel (occupier, status, notes) lives on {@see MapSolarsystemDetails}.
 *
 * @property int $id
 * @property int $map_id
 * @property int $solarsystem_id
 * @property int $map_solarsystem_details_id
 * @property string|null $alias
 * @property int $position_x
 * @property int $position_y
 * @property bool $pinned
 * @property-read int|null $signatures_count
 * @property-read int|null $uncategorized_signatures_count
 * @property-read int|null $wormhole_signatures_count
 * @property-read int|null $map_connections_count
 * @property-read Solarsystem $solarsystem
 * @property-read Map $map
 * @property-read MapSolarsystemDetails $details
 * @property-read Collection<int,MapConnection> $mapConnections
 * @property-read Collection<int,MapConnection> $connectionsTo
 * @property-read Collection<int,MapConnection> $connectionsFrom
 * @property-read WormholeSystem|null $wormholeSystem
 * @property-read Collection<int,Signature> $signatures
 * @property-read Collection<int,Signature> $wormholeSignatures
 * @property-read Collection<int,Wormhole> $wormholes
 */
#[UseFactory(MapSolarsystemFactory::class)]
#[UseEloquentBuilder(MapSolarsystemBuilder::class)]
#[CollectedBy(MapSolarsystemCollection::class)]
final class MapSolarsystem extends Model
{
    /** @use HasFactory<MapSolarsystemFactory> */
    use HasFactory;

    use HasRelationships;

    /**
     * Get all connections where this solar system is either the source or destination.
     */
    public function mapConnections(): HasManyMapConnections
    {
        $instance = $this->newRelatedInstance(MapConnection::class);

        return new HasManyMapConnections(
            $instance->newQuery(),
            $this,
        );
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
     * The persistent intel for this system on this map.
     *
     * @return BelongsTo<MapSolarsystemDetails,$this>
     */
    public function details(): BelongsTo
    {
        return $this->belongsTo(MapSolarsystemDetails::class, 'map_solarsystem_details_id');
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

    /**
     * @return HasMany<Signature,$this>
     */
    public function uncategorizedSignatures(): HasMany
    {
        return $this->hasMany(Signature::class, 'map_solarsystem_id')
            ->whereNull('signature_category_id');
    }

    /**
     * @return HasMany<Signature,$this>
     */
    public function wormholeSignatures(): HasMany
    {
        return $this->hasMany(Signature::class, 'map_solarsystem_id')
            ->whereRelation('signatureCategory', 'code', \App\Enums\SignatureCategory::Wormhole);
    }

    public function wormholes(): HasManyDeep
    {
        return $this->hasManyDeepFromRelations(
            $this->wormholeSystem(),
            new WormholeSystem()->wormholeStatics(),
            new WormholeStatic()->wormhole(),
        );
    }

    protected function casts(): array
    {
        return [
            'created_at' => 'immutable_datetime',
            'updated_at' => 'immutable_datetime',
        ];
    }
}
