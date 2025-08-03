<?php

namespace App\Http\Controllers;

use App\Enums\MassStatus;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Session;
use Illuminate\Validation\Rule;

class WatchlistController extends Controller
{
    public function update(Request $request): RedirectResponse
    {
        $request->validate([
            'allow_eol' => ['nullable', 'sometimes', 'boolean'],
            'allow_mass' => ['nullable', 'sometimes', Rule::enum(MassStatus::class)],
            'allow_eve_scout' => ['nullable', 'sometimes', 'boolean'],
        ]);

        if ($request->has('allow_eol')) {
            Session::put('allow_eol', $request->boolean('allow_eol'));
        }
        if ($request->has('allow_mass')) {
            Session::put('allow_mass', $request->enum('allow_mass', MassStatus::class));
        }
        if ($request->has('allow_eve_scout')) {
            Session::put('allow_eve_scout', $request->boolean('allow_eve_scout'));
        }

        return back()->notify('Settings updated!', 'Your watchlist settings have been successfully updated.');
    }
}
