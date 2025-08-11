<?php

declare(strict_types=1);

namespace App\Policies;

use App\Models\MapUserSetting;
use App\Models\User;

final class MapUserSettingPolicy
{
    public function update(User $user, MapUserSetting $mapUserSetting): bool
    {
        return $mapUserSetting->user()->is($user);
    }
}
