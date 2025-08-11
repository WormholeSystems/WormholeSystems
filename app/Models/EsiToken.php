<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\BelongsToMany;

/**
 * EsiToken model representing an ESI token.
 *
 * @property int $id
 * @property string $character_owner_hash
 * @property string $access_token
 * @property string|null $refresh_token
 * @property string $token_type
 * @property int $character_id
 * @property-read  CarbonImmutable $created_at
 * @property-read  CarbonImmutable $updated_at
 * @property-read  CarbonImmutable|null $expires_at
 * @property-read Character $character
 * @property-read Collection<int, EsiScope> $scopes
 */
final class EsiToken extends Model implements \NicolasKion\Esi\Interfaces\EsiToken
{
    protected $casts = [
        'expires_at' => 'immutable_datetime',
    ];

    /**
     * Get the character associated with the ESI token.
     *
     * @return BelongsTo<Character, $this>
     */
    public function character(): BelongsTo
    {
        return $this->belongsTo(Character::class);
    }

    /**
     * Get the scopes associated with the ESI token.
     *
     * @return BelongsToMany<EsiScope, $this>
     */
    public function esiScopes(): BelongsToMany
    {
        return $this->belongsToMany(EsiScope::class, 'esi_token_scope', 'esi_token_id', 'esi_scope_id');
    }

    public function isExpired(): bool
    {
        return $this->expires_at->subMinutes(5)->isPast();
    }

    public function getRefreshToken(): string
    {
        return $this->refresh_token;
    }

    public function getAccessToken(): string
    {
        return $this->access_token;
    }
}
