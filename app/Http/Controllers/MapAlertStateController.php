<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\MapAlerts\EnableMapAlertAction;
use App\Enums\MapAlertDisabledReason;
use App\Http\Requests\UpdateMapAlertStateRequest;
use App\Models\MapAlert;
use App\Models\User;
use App\Services\MapAlerts\MapAlertLifecycle;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Http\RedirectResponse;

final class MapAlertStateController extends Controller
{
    public function __construct(#[CurrentUser] private readonly User $user) {}

    public function update(UpdateMapAlertStateRequest $request, MapAlert $mapAlert, EnableMapAlertAction $enableAlert, MapAlertLifecycle $lifecycle): RedirectResponse
    {
        $enabled = $request->boolean('enabled');

        if ($enabled) {
            $error = $enableAlert->handle($mapAlert, $this->user);

            if ($error !== null) {
                return back()->notify('Alert not enabled', message: $error, type: 'error');
            }
        } else {
            $lifecycle->disable($mapAlert, $this->user, MapAlertDisabledReason::Manual);
        }

        return back()->notify('Alert state updated', message: $enabled ? 'The alert is active.' : 'The alert is disabled.');
    }
}
