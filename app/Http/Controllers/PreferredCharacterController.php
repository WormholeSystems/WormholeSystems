<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Models\Character;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;

final class PreferredCharacterController extends Controller
{
    public function __construct(
        #[CurrentUser] private readonly User $user
    ) {}

    public function store(Request $request, Character $character): RedirectResponse
    {
        // Verify the character belongs to the current user
        if ($character->user()->isNot($this->user)) {
            return back()->notify(
                'Permission denied',
                message: 'You do not have permission to set this character as preferred.',
                type: 'error'
            );
        }

        // Persist the preferred character in the database
        $this->user->preferred_character_id = $character->id;
        $this->user->save();

        // Also set as the active character for the current session
        $this->user->active_character = $character;

        return back()->notify(
            'Preferred character updated',
            message: sprintf('%s is now your preferred character.', $character->name)
        );
    }
}
