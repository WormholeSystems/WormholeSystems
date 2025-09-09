<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * @property-read int $id
 * @property int $character_id
 * @property-read Character|null $character
 * @property-read CarbonImmutable $created_at
 * @property-read CarbonImmutable $updated_at
 */
final class Audit extends \OwenIt\Auditing\Models\Audit
{
    /**
     * @return BelongsTo<Character,$this>
     */
    public function character(): BelongsTo
    {
        return $this->belongsTo(Character::class, 'character_id');
    }
}
