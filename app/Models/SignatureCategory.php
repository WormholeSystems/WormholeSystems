<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\HasMany;

/**
 * SignatureCategory model representing different categories of signatures.
 *
 * @property int $id
 * @property string $name The display name (e.g., "Wormhole", "Data Site")
 * @property string|\App\Enums\SignatureCategory $code The code identifier (e.g., "wormhole", "data")
 * @property CarbonImmutable $created_at
 * @property CarbonImmutable $updated_at
 * @property-read Collection<int, SignatureType> $signatureTypes
 * @property-read Collection<int, Signature> $signatures
 */
final class SignatureCategory extends Model
{
    protected $casts = [
        'created_at' => 'immutable_datetime',
        'updated_at' => 'immutable_datetime',
        'code' => \App\Enums\SignatureCategory::class,
    ];

    /**
     * Get all signature types that belong to this category.
     *
     * @return HasMany<SignatureType, $this>
     */
    public function signatureTypes(): HasMany
    {
        return $this->hasMany(SignatureType::class);
    }

    /**
     * Get all signatures that belong to this category.
     *
     * @return HasMany<Signature, $this>
     */
    public function signatures(): HasMany
    {
        return $this->hasMany(Signature::class);
    }
}
