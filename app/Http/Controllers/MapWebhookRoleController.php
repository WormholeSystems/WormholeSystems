<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\MapWebhookRoles\CreateMapWebhookRoleAction;
use App\Actions\MapWebhookRoles\DeleteMapWebhookRoleAction;
use App\Actions\MapWebhookRoles\UpdateMapWebhookRoleAction;
use App\Http\Requests\StoreMapWebhookRoleRequest;
use App\Http\Requests\UpdateMapWebhookRoleRequest;
use App\Models\MapWebhookRole;
use Illuminate\Http\RedirectResponse;
use Illuminate\Support\Facades\Gate;

final class MapWebhookRoleController extends Controller
{
    public function store(StoreMapWebhookRoleRequest $request, CreateMapWebhookRoleAction $action): RedirectResponse
    {
        $action->handle($request->validated());

        return back()->notify('Mention added', message: 'You can now ping it from an alert.');
    }

    public function update(UpdateMapWebhookRoleRequest $request, MapWebhookRole $mapWebhookRole, UpdateMapWebhookRoleAction $action): RedirectResponse
    {
        $action->handle($mapWebhookRole, $request->validated());

        return back()->notify('Mention updated', message: 'The mention has been updated.');
    }

    public function destroy(MapWebhookRole $mapWebhookRole, DeleteMapWebhookRoleAction $action): RedirectResponse
    {
        Gate::authorize('delete', $mapWebhookRole);

        $action->handle($mapWebhookRole);

        return back()->notify('Mention removed', message: 'Alerts that used it will keep firing without a ping.');
    }
}
