<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\MapWebhooks\CreateMapWebhookAction;
use App\Actions\MapWebhooks\DeleteMapWebhookAction;
use App\Actions\MapWebhooks\UpdateMapWebhookAction;
use App\Http\Requests\StoreMapWebhookRequest;
use App\Http\Requests\UpdateMapWebhookRequest;
use App\Http\Resources\MapInfoResource;
use App\Http\Resources\MapWebhookResource;
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
     * Show the webhooks settings page.
     */
    public function show(Map $map): Response
    {
        Gate::authorize('manageAccess', $map);

        return Inertia::render('maps/settings/ShowWebhooks', [
            'map' => $map->toResource(MapInfoResource::class),
            'webhooks' => $map->mapWebhooks()->latest()->get()->toResourceCollection(MapWebhookResource::class),
            'permission' => $map->getUserPermission($this->user)?->value,
        ]);
    }

    /**
     * @throws Throwable
     */
    public function store(StoreMapWebhookRequest $request, CreateMapWebhookAction $action): RedirectResponse
    {
        $action->handle($request->validated());

        return back()->notify('Webhook created', message: 'You will be notified in Discord when the trigger fires.');
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

        $action->handle($mapWebhook);

        return back()->notify('Webhook deleted', message: 'The webhook has been removed.');
    }
}
