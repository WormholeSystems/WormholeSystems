<?php

declare(strict_types=1);

namespace App\Models;

use App\Enums\MapAlertEventAction;
use Carbon\CarbonImmutable;
use Database\Factories\MapAlertEventFactory;
use Illuminate\Database\Eloquent\Attributes\UseFactory;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use LogicException;

/**
 * @property int $id
 * @property int|null $map_alert_id
 * @property int $map_id
 * @property int|null $actor_user_id
 * @property string|null $actor_name
 * @property MapAlertEventAction $action
 * @property array<string, mixed> $snapshot
 * @property string|null $reason
 * @property CarbonImmutable $created_at
 */
#[UseFactory(MapAlertEventFactory::class)]
final class MapAlertEvent extends Model
{
    /** @use HasFactory<MapAlertEventFactory> */
    use HasFactory;

    public const UPDATED_AT = null;

    /** @return BelongsTo<MapAlert, $this> */
    public function alert(): BelongsTo
    {
        return $this->belongsTo(MapAlert::class, 'map_alert_id');
    }

    /** @return BelongsTo<Map, $this> */
    public function map(): BelongsTo
    {
        return $this->belongsTo(Map::class);
    }

    /** @return BelongsTo<User, $this> */
    public function actor(): BelongsTo
    {
        return $this->belongsTo(User::class, 'actor_user_id');
    }

    protected static function booted(): void
    {
        self::updating(function (): never {
            throw new LogicException('Map alert events are immutable.');
        });

        self::deleting(function (): never {
            throw new LogicException('Map alert events are immutable.');
        });
    }

    /** @return array<string, string> */
    protected function casts(): array
    {
        return [
            'action' => MapAlertEventAction::class,
            'snapshot' => 'array',
            'created_at' => 'immutable_datetime',
        ];
    }
}
