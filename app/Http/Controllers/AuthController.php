<?php

namespace App\Http\Controllers;

use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Session;

class AuthController
{
    public function __construct(
        #[CurrentUser] protected ?User $user = null
    ) {}

    public function show(Request $request): RedirectResponse
    {
        if ($request->boolean('add_to_account') && $this->user) {
            Session::put('add_to_account', $this->user->id);
        }

        return to_route('eve.show');
    }
}
