<?php

declare(strict_types=1);

namespace App\Models;

use Illuminate\Database\Eloquent\Model;

/**
 * @property int $id
 * @property int $players
 * @property string $server_version
 * @property string $start_time
 * @property bool $vip
 */
final class ServerStatus extends Model
{
    protected function casts(): array
    {
        return [
            'start_time' => 'datetime',
            'vip' => 'boolean',
        ];
    }
}
