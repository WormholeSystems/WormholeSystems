<?php

declare(strict_types=1);

namespace App\Models;

use App\Casts\SolarsystemClassArrayCast;
use App\Enums\SolarsystemClass;
use App\Enums\WormholeSignature;
use App\Models\SignatureCategory as SignatureCategoryModel;
use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;

/**
 * SignatureType model representing different types of signatures that can appear in the game.
 *
 * @property int $id
 * @property string $name The display name of the signature type (e.g., "Perimeter Ambush Point", "K162 - C1")
 * @property WormholeSignature|null $signature The signature code for wormholes (e.g., "K162", "A009")
 * @property int $signature_category_id Foreign key to signature_categories table
 * @property SolarsystemClass|null $target_class For wormholes, the target solarsystem class (e.g., "1", "hs", "ls", "ns")
 * @property string|null $extra Additional information (e.g., "Thera", "Barbican", "Pochven")
 * @property array<SolarsystemClass>|null $spawn_areas Array of solarsystem classes where this signature can appear
 * @property CarbonImmutable $created_at
 * @property CarbonImmutable $updated_at
 * @property-read SignatureCategoryModel $category The category this signature type belongs to
 * @property-read Collection<int, Signature> $signatures
 * @property-read Wormhole|null $wormhole The wormhole associated with this signature type, if applicable
 */
final class SignatureType extends Model
{
    protected $fillable = [
        'name',
        'signature',
        'signature_category_id',
        'target_class',
        'extra',
        'spawn_areas',
    ];

    protected $casts = [
        'signature' => WormholeSignature::class,
        'target_class' => SolarsystemClass::class,
        'spawn_areas' => SolarsystemClassArrayCast::class,
        'created_at' => 'immutable_datetime',
        'updated_at' => 'immutable_datetime',
    ];

    /**
     * Get the category this signature type belongs to.
     *
     * @return BelongsTo<SignatureCategoryModel, $this>
     */
    public function category(): BelongsTo
    {
        return $this->belongsTo(SignatureCategoryModel::class, 'signature_category_id');
    }

    /**
     * Get all signatures that use this signature type.
     *
     * @return HasMany<Signature, $this>
     */
    public function signatures(): HasMany
    {
        return $this->hasMany(Signature::class);
    }

    public function wormhole(): BelongsTo
    {
        return $this->belongsTo(Wormhole::class, 'signature', 'name');
    }
}
