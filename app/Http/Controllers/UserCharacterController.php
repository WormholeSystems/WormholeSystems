<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Models\Character;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;

class UserCharacterController extends Controller
{
    public function __construct(#[CurrentUser] protected User $user) {}

    public function delete(Request $request, Character $character): RedirectResponse
    {
        if (! $character->user()->is($request->user())) {
            to_route('home')->notify('Character not removed', type: 'error', message: 'You do not have permission to remove this character.');
        }

        if ($request->user()->characters()->count() === 1) {
            return to_route('home')->notify('Character not removed', type: 'error', message: 'You cannot remove your last character.');
        }

        $character->user()->disassociate();
        $character->save();

        $this->user->setActiveCharacter($this->user->characters()->first());

        return to_route('home')->notify('Character removed', message: 'You have successfully removed the character from your account.');
    }

    public function update(Request $request, Character $character): RedirectResponse
    {
        abort_if($character->user()->isNot($request->user()), 403);

        $request->user()->setActiveCharacter($character);

        return back()->notify('Character changed', message: sprintf('You are now acting as %s', $character->name));
    }
}
