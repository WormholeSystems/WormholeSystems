<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Model;

/**
 * Flag model representing a flag in the game.
 *
 * @property int $id
 * @property string $name
 * @property string $text
 * @property int $order_id
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 */
class Flag extends Model
{
    public $incrementing = false;
}
