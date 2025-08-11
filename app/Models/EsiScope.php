<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsToMany;

/**
 * EsiScope model representing a scope in the ESI (EVE Swagger Interface).
 *
 * @property int $id
 * @property string $name
 * @property bool $is_default
 * @property-read CarbonImmutable $created_at
 * @property-read CarbonImmutable $updated_at
 */
final class EsiScope extends Model
{
    /**
     * @return BelongsToMany<EsiToken,$this>
     */
    public function esiTokens(): BelongsToMany
    {
        return $this->belongsToMany(EsiToken::class, 'esi_token_scope', 'esi_scope_id', 'esi_token_id');
    }

    protected function casts(): array
    {
        return [
            'is_default' => 'boolean',
        ];
    }
}
