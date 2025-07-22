<?php

namespace App\Http\Middleware;

use App\Http\Resources\CharacterResource;
use App\Http\Resources\UserResource;
use App\Models\User;
use App\Scopes\CharacterDoesntHaveRequiredScopes;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Foundation\Inspiring;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;
use Inertia\Inertia;
use Inertia\Middleware;
use NicolasKion\Esi\Enums\EsiScope;
use Throwable;
use Tighten\Ziggy\Ziggy;

class HandleInertiaRequests extends Middleware
{
    public function __construct(#[CurrentUser] protected ?User $user = null) {}

    /**
     * The root template that's loaded on the first page visit.
     *
     * @see https://inertiajs.com/server-side-setup#root-template
     *
     * @var string
     */
    protected $rootView = 'app';

    /**
     * Determines the current asset version.
     *
     * @see https://inertiajs.com/asset-versioning
     */
    public function version(Request $request): ?string
    {
        return parent::version($request);
    }

    /**
     * Define the props that are shared by default.
     *
     * @see https://inertiajs.com/shared-data
     *
     * @return array<string, mixed>
     *
     * @throws Throwable
     */
    public function share(Request $request): array
    {
        [$message, $author] = str(Inspiring::quotes()->random())->explode('-');

        return [
            ...parent::share($request),
            'name' => config('app.name'),
            'quote' => ['message' => trim((string) $message), 'author' => trim((string) $author)],
            'auth' => [
                'user' => $this->user?->toResource(UserResource::class),
            ],
            'ziggy' => [
                ...(new Ziggy)->toArray(),
                'location' => $request->url(),
            ],
            'sidebarOpen' => ! $request->hasCookie('sidebar_state') || $request->cookie('sidebar_state') === 'true',
            'notification' => Inertia::always(
                $request->session()->get('notification')
            ),
            'missing_scopes' => $this->getMissingScopes(),
        ];
    }

    /**
     * @throws Throwable
     */
    public function getMissingScopes(): JsonResource
    {
        if (! $this->user instanceof \App\Models\User) {
            return new JsonResource([]);
        }

        $characters_with_missing_scopes = $this->user->characters()
            ->tap(new CharacterDoesntHaveRequiredScopes([
                EsiScope::ReadOnlineStatus,
                EsiScope::ReadLocations,
                EsiScope::ReadShip,
            ]))
            ->get();

        return $characters_with_missing_scopes->toResourceCollection(CharacterResource::class);
    }
}
