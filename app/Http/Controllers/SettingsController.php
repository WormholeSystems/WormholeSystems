<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Http\Resources\CharacterResource;
use App\Http\Resources\MapAlertResource;
use App\Http\Resources\TokenResource;
use App\Models\User;
use App\Services\Discord\DiscordInvite;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Session;
use Inertia\Inertia;
use Inertia\Response;
use Throwable;

final class SettingsController extends Controller
{
    public function __construct(
        #[CurrentUser] private readonly User $user,
        private readonly DiscordInvite $discordInvite,
    ) {}

    /** @throws Throwable */
    public function show(Request $request): Response
    {
        $characters = $this->user->characters()->with('esiScopes')->get();
        $discordAccount = $this->user->discordAccount()->first();
        $discordAlerts = $this->user->createdMapAlerts()
            ->bot()
            ->with(['map:id,name', 'targetSolarsystem:id,name', 'originSolarsystem:id,name'])
            ->latest()
            ->get();

        return Inertia::render('settings/Show', [
            'section' => in_array($request->query('section'), ['esi', 'discord', 'tokens'], true)
                ? $request->query('section')
                : 'esi',
            'characters' => $characters->toResourceCollection(CharacterResource::class),
            'discordAccount' => $discordAccount,
            'discordAlerts' => $discordAlerts->toResourceCollection(MapAlertResource::class),
            'tokens' => $this->user->tokens()->get()->toResourceCollection(TokenResource::class),
            'token' => Session::get('token'),
            'discordConfigured' => filled(config('services.discord.client_id')),
            'discordInviteUrl' => $this->discordInvite->url(),
            'pendingDiscordAccount' => is_string($pendingToken = Session::get('pending_discord_link_token'))
                ? Cache::get('discord.link.'.$pendingToken)
                : null,
        ]);
    }
}
