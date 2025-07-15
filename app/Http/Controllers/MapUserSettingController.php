<?php

namespace App\Http\Controllers;

use App\Actions\UpdateMapUserSettingRequestAction;
use App\Http\Requests\UpdateMapUserSettingRequest;
use App\Models\MapUserSetting;
use Throwable;

class MapUserSettingController extends Controller
{
    /**
     * @throws Throwable
     */
    public function update(UpdateMapUserSettingRequest $request, MapUserSetting $mapUserSetting, UpdateMapUserSettingRequestAction $action)
    {
        $action->handle($mapUserSetting, $request->validated());

        return back()->notify('Settings updated!', 'Your map user settings have been successfully updated.');
    }
}
