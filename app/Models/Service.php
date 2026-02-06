<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsToMany;

/**
 * Service model representing a station service.
 *
 * @property int $id
 * @property string $name
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Collection<int,StationOperation> $operations
 */
final class Service extends Model
{
    public $incrementing = false;

    protected $fillable = [
        'id',
        'name',
    ];

    /**
     * @return BelongsToMany<StationOperation,$this>
     */
    public function operations(): BelongsToMany
    {
        return $this->belongsToMany(
            StationOperation::class,
            'operation_services',
            'service_id',
            'station_operation_id'
        );
    }
}
