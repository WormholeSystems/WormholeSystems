<?php

declare(strict_types=1);

namespace App\Policies;

use App\Enums\Permission;
use App\Models\Map;
use App\Models\User;
use App\Services\AffiliationWhitelist;
use Illuminate\Container\Attributes\Config;

final class MapPolicy
{
    /**
     * @param  list<int>  $mapCreatorAffiliationIds
     */
    public function __construct(
        #[Config('access.map_creator_affiliation_ids')]
        private array $mapCreatorAffiliationIds = [],
    ) {}

    public function viewAny(): bool
    {
        return true;
    }

    public function view(?User $user, Map $map): bool
    {
        if ($map->is_public) {
            return true;
        }

        if ($map->share_token !== null) {
            $providedToken = request()->query('share_token')
                ?? session("map_share_token_{$map->id}");

            if ($providedToken === $map->share_token) {
                return true;
            }
        }

        if (! $user instanceof User) {
            return false;
        }

        return $map->mapAccessors()->notExpired()->whereIn('accessible_id', $user->getAccessibleIds())->exists();
    }

    public function viewCharacters(?User $user, Map $map): bool
    {
        if (! $user instanceof User) {
            return false;
        }

        $permission = $map->getUserPermission($user);

        return $permission instanceof Permission && $permission->isAtLeast(Permission::Member);
    }

    /**
     * Creating a map is open to every authenticated user unless the instance
     * names the affiliations allowed to do so. Same shape as the login
     * whitelist, and independent of it: an instance can be open to an alliance
     * while only its leadership creates maps.
     *
     * The check is against the *active* character rather than every character
     * on the account: an account holding a director and an alt in an NPC
     * corporation should not let the alt create maps, and the active character
     * is the one the map is created as.
     */
    public function create(?User $user): bool
    {
        if (! $user instanceof User) {
            return false;
        }

        $whitelist = new AffiliationWhitelist($this->mapCreatorAffiliationIds);

        // Nothing configured means nothing changes: return before resolving the
        // active character, which is work an unconfigured instance should not do.
        if (! $whitelist->isEnforced()) {
            return true;
        }

        $character = $user->active_character;

        return $whitelist->allows([
            $character?->id,
            $character?->corporation_id,
            $character?->alliance_id,
        ]);
    }

    public function update(?User $user, Map $map): bool
    {
        if (! $user instanceof User) {
            return false;
        }

        $permission = $map->getUserPermission($user);

        return $permission instanceof Permission && $permission->isAtLeast(Permission::Member);
    }

    public function manageAccess(?User $user, Map $map): bool
    {
        if (! $user instanceof User) {
            return false;
        }

        $permission = $map->getUserPermission($user);

        return $permission instanceof Permission && $permission->isAtLeast(Permission::Manager);
    }

    /**
     * Manager-level map configuration that affects every viewer (e.g. the layout mode).
     */
    public function updateSettings(?User $user, Map $map): bool
    {
        if (! $user instanceof User) {
            return false;
        }

        $permission = $map->getUserPermission($user);

        return $permission instanceof Permission && $permission->isAtLeast(Permission::Manager);
    }

    public function delete(?User $user, Map $map): bool
    {
        if (! $user instanceof User) {
            return false;
        }

        return $map->mapAccessors()
            ->notExpired()
            ->whereIn('accessible_id', $user->getAccessibleIds())
            ->where('is_owner', true)
            ->exists();
    }
}
