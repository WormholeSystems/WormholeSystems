<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Enums\MapAlertDeliveryType;
use App\Enums\MapAlertDisabledReason;
use App\Enums\MapAlertMentionMode;
use App\Models\DiscordAccount;
use App\Models\MapAlert;
use App\Models\User;
use App\Services\MapAlerts\MapAlertLifecycle;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\DB;
use Laravel\Socialite\Facades\Socialite;
use Laravel\Socialite\Two\InvalidStateException;
use SocialiteProviders\Discord\Provider as DiscordProvider;
use SocialiteProviders\Manager\OAuth2\User as DiscordUser;
use Symfony\Component\HttpFoundation\Response;

final class DiscordAccountController extends Controller
{
    public function __construct(
        #[CurrentUser] private readonly User $user,
        private readonly MapAlertLifecycle $lifecycle,
    ) {}

    public function redirect(): RedirectResponse
    {
        abort_unless(filled(config('services.discord.client_id')), Response::HTTP_SERVICE_UNAVAILABLE, 'Discord linking is not configured.');

        $provider = Socialite::driver('discord');
        assert($provider instanceof DiscordProvider);

        return $provider
            ->setScopes(['identify'])
            ->withConsent()
            ->with(['prompt' => 'consent'])
            ->redirect();
    }

    public function callback(Request $request): RedirectResponse
    {
        if ($request->filled('error') || $request->missing('code')) {
            return to_route('settings.show', ['section' => 'discord'])
                ->notify('Discord not connected', message: 'Discord authorization was cancelled.', type: 'error');
        }

        try {
            $identity = Socialite::driver('discord')->user();
        } catch (InvalidStateException) {
            abort(Response::HTTP_FORBIDDEN);
        }

        assert($identity instanceof DiscordUser);
        $raw = $identity->getRaw();
        $this->linkIdentity([
            'id' => (string) $identity->getId(),
            'username' => (string) ($raw['username'] ?? $identity->getNickname()),
            'global_name' => $raw['global_name'] ?? null,
            'avatar' => $raw['avatar'] ?? null,
        ]);
        session()->forget('pending_discord_link_token');

        return to_route('settings.show', ['section' => 'discord'])->notify('Discord connected', 'Your Discord account is now linked.');
    }

    public function link(string $token): RedirectResponse
    {
        $identity = Cache::get('discord.link.'.$token);
        abort_unless(is_array($identity), Response::HTTP_NOT_FOUND, 'This Discord link has expired.');

        session(['pending_discord_link_token' => $token]);

        return to_route('settings.show', ['section' => 'discord']);
    }

    public function confirmLink(): RedirectResponse
    {
        $token = session()->pull('pending_discord_link_token');
        $identity = is_string($token) ? Cache::pull('discord.link.'.$token) : null;
        abort_unless(is_array($identity), Response::HTTP_UNPROCESSABLE_ENTITY, 'There is no Discord account waiting to be linked.');

        $this->linkIdentity($identity);

        return to_route('settings.show', ['section' => 'discord'])->notify('Discord connected', 'Your Discord account is now linked.');
    }

    public function destroy(): RedirectResponse
    {
        DB::transaction(function (): void {
            $this->user->createdMapAlerts()
                ->where('is_active', true)
                ->where(function ($query): void {
                    $query->where('delivery_type', MapAlertDeliveryType::DiscordDm)
                        ->orWhere(function ($query): void {
                            $query->where('delivery_type', MapAlertDeliveryType::DiscordChannel)
                                ->where('mention_mode', MapAlertMentionMode::Creator);
                        });
                })
                ->each(
                    fn (MapAlert $alert) => $this->lifecycle->disable($alert, $this->user, MapAlertDisabledReason::DiscordAccountDisconnected),
                );
            $this->user->discordAccount()->delete();
        });
        session()->forget('pending_discord_link_token');

        return back()->notify('Discord disconnected', 'Alerts that mention your Discord account were disabled.');
    }

    /** @param array{id: string, username: string, global_name?: string|null, avatar?: string|null} $identity */
    private function linkIdentity(array $identity): void
    {
        abort_if(DiscordAccount::query()->where('discord_user_id', $identity['id'])->where('user_id', '!=', $this->user->id)->exists(), Response::HTTP_CONFLICT, 'This Discord account is already linked.');

        $this->user->discordAccount()->updateOrCreate([], [
            'discord_user_id' => $identity['id'],
            'username' => $identity['username'],
            'display_name' => $identity['global_name'] ?? null,
            'avatar' => $identity['avatar'] ?? null,
        ]);
    }
}
