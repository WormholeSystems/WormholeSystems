<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Session;

final class AuthController
{
    public function __construct(
        #[CurrentUser] private ?User $user = null
    ) {}

    public function show(Request $request): RedirectResponse
    {
        if ($request->boolean('add_to_account') && $this->user) {
            Session::put('add_to_account', $this->user->id);
        }

        return to_route('eve.show');
    }

    public function destroy(): RedirectResponse
    {
        Auth::logout();
        Session::invalidate();
        Session::regenerateToken();

        return to_route('login')->notify(
            'Logged out successfully.',
            'You have been logged out of your account.'
        );
    }
}
