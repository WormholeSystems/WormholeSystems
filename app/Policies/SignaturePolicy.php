<?php

declare(strict_types=1);

namespace App\Policies;

use App\Models\Signature;
use App\Models\User;

final class SignaturePolicy
{
    /**
     * Create a new policy instance.
     */
    public function __construct()
    {
        //
    }

    public function update(User $user, Signature $signature): bool
    {
        return $user->can('update', $signature->mapSolarsystem->map);
    }

    public function delete(User $user, Signature $signature): bool
    {
        return $user->can('update', $signature->mapSolarsystem->map);
    }
}
