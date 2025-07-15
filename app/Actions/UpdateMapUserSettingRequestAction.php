<?php

declare(strict_types=1);

namespace App\Actions;

use App\Events\Characters\CharacterStatusUpdatedEvent;
use App\Models\MapUserSetting;
use Illuminate\Support\Facades\DB;
use Throwable;

final readonly class UpdateMapUserSettingRequestAction
{
    /**
     * @throws Throwable
     */
    public function handle(MapUserSetting $mapUserSetting, array $data): MapUserSetting
    {
        return DB::transaction(function () use ($data, $mapUserSetting): MapUserSetting {
            $mapUserSetting->update($data);

            broadcast(new CharacterStatusUpdatedEvent($mapUserSetting->map_id))->toOthers();

            return $mapUserSetting;
        });
    }
}
