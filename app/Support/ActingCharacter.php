<?php

declare(strict_types=1);

namespace App\Support;

use App\Models\Character;
use App\Models\User;
use Illuminate\Support\Facades\Session;

final readonly class ActingCharacter
{
    /**
     * Best-effort "who is acting" for auditing. Read-only and never throws: statistics
     * must not be able to break a signature write. Mirrors User::active_character's
     * lookup order (session, then preferred, then any) minus every side effect.
     */
    public static function resolve(?User $user): ?Character
    {
        if (! $user instanceof User) {
            return null;
        }

        $id = Session::get(User::SESSION_ACTIVE_CHARACTER_ID) ?? $user->preferred_character_id;

        return ($id ? $user->characters->find($id) : null) ?? $user->characters->first();
    }
}
