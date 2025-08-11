<?php

declare(strict_types=1);

namespace App\Http\Middleware;

use App\Http\Resources\CharacterResource;
use App\Http\Resources\UserResource;
use App\Models\ServerStatus;
use App\Models\User;
use App\Scopes\CharacterDoesntHaveRequiredScopes;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;
use Illuminate\Support\Facades\Cookie;
use Inertia\Inertia;
use Inertia\Middleware;
use NicolasKion\Esi\Enums\EsiScope;
use Throwable;

final class HandleInertiaRequests extends Middleware
{
    /**
     * The root template that's loaded on the first page visit.
     *
     * @see https://inertiajs.com/server-side-setup#root-template
     *
     * @var string
     */
    protected $rootView = 'app';

    public function __construct(#[CurrentUser] protected ?User $user = null) {}

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
        return [
            ...parent::share($request),
            'name' => config('app.name'),
            'auth' => [
                'user' => $this->user?->toResource(UserResource::class),
            ],
            'notification' => Inertia::always(
                $request->session()->get('notification')
            ),
            'missing_scopes' => $this->getMissingScopes(),
            'discord' => fn (): array => [
                'invite' => config('services.discord.invite'),
            ],
            'layout' => $this->getLayout($request),
            'server_status' => fn () => ServerStatus::query()->latest()->first(),
        ];
    }

    /**
     * @throws Throwable
     */
    public function getMissingScopes(): JsonResource
    {
        if (! $this->user instanceof User) {
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

    private function getLayout(Request $request): array
    {
        $cookie = $request->cookie('layout');

        if ($cookie) {
            return $this->parseLayoutCookie($cookie);
        }

        $layout = [
            'map_height' => 1_000,
            'scale' => 1.0,
        ];

        Cookie::make('layout', json_encode($layout), 60 * 24 * 365); // 1 year

        return $layout;
    }

    private function parseLayoutCookie(string $cookie): array
    {
        try {
            $layout = json_decode($cookie, true, 512, JSON_THROW_ON_ERROR);
        } catch (Throwable) {
            return $this->createLayoutCookie();
        }

        return [
            'map_height' => $layout['map_height'] ?? 1_000,
            'scale' => $layout['scale'] ?? 1.0,
        ];
    }

    private function createLayoutCookie(): array
    {
        $layout = [
            'map_height' => 1_000,
            'scale' => 1.0,
        ];

        Cookie::make('layout', json_encode($layout), 60 * 24 * 365); // 1 year

        return $layout;
    }
}
