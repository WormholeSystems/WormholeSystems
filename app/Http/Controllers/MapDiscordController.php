<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Enums\MapAlertEventAction;
use App\Http\Resources\MapAlertEventResource;
use App\Http\Resources\MapAlertResource;
use App\Http\Resources\MapInfoResource;
use App\Http\Resources\MapWebhookResource;
use App\Http\Resources\MapWebhookRoleResource;
use App\Models\Map;
use App\Models\MapAlertEvent;
use App\Models\User;
use App\Services\Discord\DiscordInvite;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Gate;
use Inertia\Inertia;
use Inertia\Response;

final class MapDiscordController extends Controller
{
    public function __construct(
        #[CurrentUser] private readonly User $user,
        private readonly DiscordInvite $discordInvite,
    ) {}

    public function show(Request $request, Map $map): Response
    {
        Gate::authorize('manageAccess', $map);

        return Inertia::render('maps/settings/Discord', [
            'tab' => $request->query('tab') === 'webhooks' ? 'webhooks' : 'bot',
            'map' => $map->toResource(MapInfoResource::class),
            'webhooks' => fn () => $map->mapWebhooks()->withCount('alerts')->latest()->get()->toResourceCollection(MapWebhookResource::class),
            'roles' => fn () => $map->mapWebhookRoles()->withCount('alerts')->latest()->get()->toResourceCollection(MapWebhookRoleResource::class),
            'alerts' => fn () => $map->mapAlerts()->webhook()->with(['webhook', 'originSolarsystem'])->latest()->get()->toResourceCollection(MapAlertResource::class),
            'botAlerts' => fn () => $map->mapAlerts()
                ->bot()
                ->with(['creator.characters', 'targetSolarsystem', 'originSolarsystem'])
                ->latest()
                ->get()
                ->toResourceCollection(MapAlertResource::class),
            'alertEvents' => fn () => MapAlertEvent::query()
                ->where('map_id', $map->id)
                ->whereIn('action', [MapAlertEventAction::Disabled, MapAlertEventAction::Removed])
                ->with('actor.characters')
                ->latest()
                ->limit(25)
                ->get()
                ->toResourceCollection(MapAlertEventResource::class),
            'permission' => $map->getUserPermission($this->user)?->value,
            'discordInviteUrl' => $this->discordInvite->url(),
        ]);
    }
}
