<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * OperationService model representing the pivot between operations and services.
 *
 * @property int $id
 * @property int $station_operation_id
 * @property int $service_id
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read StationOperation $operation
 * @property-read Service $service
 */
final class OperationService extends Model
{
    protected $fillable = [
        'station_operation_id',
        'service_id',
    ];

    /**
     * @return BelongsTo<StationOperation,$this>
     */
    public function operation(): BelongsTo
    {
        return $this->belongsTo(StationOperation::class, 'station_operation_id');
    }

    /**
     * @return BelongsTo<Service,$this>
     */
    public function service(): BelongsTo
    {
        return $this->belongsTo(Service::class);
    }
}
