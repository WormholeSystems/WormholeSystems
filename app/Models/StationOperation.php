<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsToMany;
use Illuminate\Database\Eloquent\Relations\HasMany;

/**
 * StationOperation model representing a station operation type.
 *
 * @property int $id
 * @property string $name
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Collection<int,Service> $services
 * @property-read Collection<int,Station> $stations
 */
final class StationOperation extends Model
{
    public $incrementing = false;

    protected $fillable = [
        'id',
        'name',
    ];

    /**
     * @return BelongsToMany<Service,$this>
     */
    public function services(): BelongsToMany
    {
        return $this->belongsToMany(
            Service::class,
            'operation_services',
            'station_operation_id',
            'service_id'
        );
    }

    /**
     * @return HasMany<Station,$this>
     */
    public function stations(): HasMany
    {
        return $this->hasMany(Station::class, 'operation_id');
    }
}
