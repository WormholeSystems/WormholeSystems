<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Services\EsiAuthService;
use Illuminate\Http\Client\ConnectionException;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Session;
use Laravel\Socialite\Facades\Socialite;
use NicolasKion\Esi\Enums\EsiScope;
use SocialiteProviders\Eveonline\Provider;

class EveController extends Controller
{
    /**
     * @throws ConnectionException
     */
    public function store(EsiAuthService $esiAuthService): RedirectResponse
    {
        $account_id = Session::get('add_to_account');

        [$user, $character] = $esiAuthService->getUser($account_id);

        if (! $user) {
            return to_route('home')->notify('Error', message: 'Unable to retrieve user information from EVE Online. Please try again later.');
        }

        Auth::login($user, remember: true);
        $user->setActiveCharacter($character);

        if ($account_id) {
            return to_route('home')->notify('Account Updated', message: 'Your character has been added to your account.');
        }

        $redirect = redirect()->intended(route('home'))->getTargetUrl();

        return redirect($redirect)->notify('Welcome back!', message: sprintf('We have successfully logged you in, %s.', $character->name));
    }

    public function show(Request $request): RedirectResponse
    {
        if ($request->query('add_to_account')) {
            $request->session()->put('add_to_account', auth()->id());

        }

        $eve_provider = Socialite::driver('eveonline');

        assert($eve_provider instanceof Provider);

        if ($request->boolean('without_scopes')) {
            return $eve_provider
                ->redirect();
        }

        return $eve_provider
            ->scopes(
                $request->scopes ?
                    EsiScope::fromRequest($request->scopes)
                    : \App\Models\EsiScope::query()->where('is_default', true)->pluck('name')->toArray()
            )
            ->redirect();
    }
}
