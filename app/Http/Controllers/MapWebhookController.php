<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\MapWebhooks\CreateMapWebhookAction;
use App\Actions\MapWebhooks\DeleteMapWebhookAction;
use App\Actions\MapWebhooks\UpdateMapWebhookAction;
use App\Http\Requests\StoreMapWebhookRequest;
use App\Http\Requests\UpdateMapWebhookRequest;
use App\Http\Resources\MapAlertResource;
use App\Http\Resources\MapInfoResource;
use App\Http\Resources\MapWebhookResource;
use App\Http\Resources\MapWebhookRoleResource;
use App\Models\Map;
use App\Models\MapWebhook;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Http\RedirectResponse;
use Illuminate\Support\Facades\Gate;
use Inertia\Inertia;
use Inertia\Response;
use Throwable;

final class MapWebhookController extends Controller
{
    public function __construct(
        #[CurrentUser] private readonly User $user,
    ) {}

    /**
     * Show the alerts settings page: webhook destinations, roles, and the alerts that use them.
     */
    public function show(Map $map): Response
    {
        Gate::authorize('manageAccess', $map);

        return Inertia::render('maps/settings/ShowWebhooks', [
            'map' => $map->toResource(MapInfoResource::class),
            'webhooks' => $map->mapWebhooks()->withCount('alerts')->latest()->get()->toResourceCollection(MapWebhookResource::class),
            'roles' => $map->mapWebhookRoles()->withCount('alerts')->latest()->get()->toResourceCollection(MapWebhookRoleResource::class),
            'alerts' => $map->mapAlerts()->latest()->get()->toResourceCollection(MapAlertResource::class),
            'permission' => $map->getUserPermission($this->user)?->value,
        ]);
    }

    /**
     * @throws Throwable
     */
    public function store(StoreMapWebhookRequest $request, CreateMapWebhookAction $action): RedirectResponse
    {
        $action->handle($request->validated());

        return back()->notify('Webhook created', message: 'You can now point alerts at this Discord channel.');
    }

    /**
     * @throws Throwable
     */
    public function update(UpdateMapWebhookRequest $request, MapWebhook $mapWebhook, UpdateMapWebhookAction $action): RedirectResponse
    {
        $data = $request->validated();

        if (blank($data['discord_webhook_url'] ?? null)) {
            unset($data['discord_webhook_url']);
        }

        $action->handle($mapWebhook, $data);

        return back()->notify('Webhook updated', message: 'The webhook has been updated.');
    }

    /**
     * @throws Throwable
     */
    public function destroy(MapWebhook $mapWebhook, DeleteMapWebhookAction $action): RedirectResponse
    {
        Gate::authorize('delete', $mapWebhook);

        if ($mapWebhook->alerts()->exists()) {
            return back()->notify(
                'Webhook in use',
                message: 'Remove or reassign its alerts before deleting this webhook.',
                type: 'error',
            );
        }

        $action->handle($mapWebhook);

        return back()->notify('Webhook deleted', message: 'The webhook has been removed.');
    }
}
