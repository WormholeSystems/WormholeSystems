<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\MapAlerts\CreateMapAlertAction;
use App\Actions\MapAlerts\DeleteMapAlertAction;
use App\Actions\MapAlerts\UpdateMapAlertAction;
use App\Http\Requests\StoreMapAlertRequest;
use App\Http\Requests\UpdateMapAlertRequest;
use App\Models\MapAlert;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Http\RedirectResponse;
use Illuminate\Support\Facades\Gate;

final class MapAlertController extends Controller
{
    public function __construct(#[CurrentUser] private readonly User $user) {}

    public function store(StoreMapAlertRequest $request, CreateMapAlertAction $action): RedirectResponse
    {
        $action->handle($request->validated(), $this->user);

        return back()->notify('Alert created', message: 'You will be notified in Discord when the trigger fires.');
    }

    public function update(UpdateMapAlertRequest $request, MapAlert $mapAlert, UpdateMapAlertAction $action): RedirectResponse
    {
        $action->handle($mapAlert, $request->validated(), $this->user);

        return back()->notify('Alert updated', message: 'The alert has been updated.');
    }

    public function destroy(MapAlert $mapAlert, DeleteMapAlertAction $action): RedirectResponse
    {
        Gate::authorize('delete', $mapAlert);

        $action->handle($mapAlert, $this->user);

        return back()->notify('Alert deleted', message: 'The alert has been removed.');
    }
}
