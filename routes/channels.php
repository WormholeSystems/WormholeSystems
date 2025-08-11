<?php

declare(strict_types=1);

use App\Models\Map;
use App\Models\User;
use Illuminate\Support\Facades\Broadcast;

Broadcast::channel('Map.{map}', function (User $user, Map $map) {
    return $user->can('view', $map);
});
