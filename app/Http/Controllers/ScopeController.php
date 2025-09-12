<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Http\Resources\CharacterResource;
use App\Models\Character;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Gate;
use Inertia\Inertia;
use Inertia\Response;
use Session;
use Throwable;

use function back;

final class ScopeController extends Controller
{
    public function __construct(
        #[CurrentUser] private readonly User $user
    ) {}

    /**
     * @throws Throwable
     */
    public function index(): Response
    {
        $characters = $this->user->characters()->with('esiScopes')->get()->toResourceCollection(CharacterResource::class);

        return Inertia::render('scopes/ShowAllScopes', [
            'characters' => $characters,
        ]);
    }

    public function show(Request $request): RedirectResponse
    {
        Session::put('redirect_to', route('scopes.index'));

        return to_route('eve.show', [
            'add_to_account' => true,
            'scopes' => $request->input('scopes'),
        ]);
    }

    public function destroy(Character $character): RedirectResponse
    {
        Gate::denyIf($character->user()->isNot($this->user));

        $character->esiTokens()->delete();

        return back()->notify('Scopes removed successfully.', message: 'We have removed all scopes for this character.');
    }
}
