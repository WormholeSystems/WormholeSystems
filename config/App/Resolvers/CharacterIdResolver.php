<?php

declare(strict_types=1);

namespace App\Resolvers;

use Illuminate\Support\Facades\Auth;
use OwenIt\Auditing\Contracts\Auditable;
use OwenIt\Auditing\Contracts\Resolver;

final class CharacterIdResolver implements Resolver
{
    public static function resolve(?Auditable $auditable = null)
    {
        $user = Auth::user();

        return $user?->active_character->id;
    }
}
