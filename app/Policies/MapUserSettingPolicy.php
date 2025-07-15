<?php

namespace App\Policies;

use App\Models\MapUserSetting;
use App\Models\User;

class MapUserSettingPolicy
{
    public function update(User $user, MapUserSetting $mapUserSetting): bool
    {
        return $mapUserSetting->user()->is($user);
    }
}
