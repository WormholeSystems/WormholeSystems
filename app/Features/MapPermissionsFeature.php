<?php

declare(strict_types=1);

namespace App\Features;

use App\Enums\Permission;
use App\Models\Character;
use App\Models\Map;
use App\Models\User;
use Inertia\ProvidesInertiaProperties;
use Inertia\RenderContext;

final readonly class MapPermissionsFeature implements ProvidesInertiaProperties
{
    public function __construct(
        private Map $map,
        private ?User $user,
    ) {}

    public function toInertiaProperties(RenderContext $context): array
    {
        $permission = $this->user instanceof User
            ? $this->map->getUserPermission($this->user)
            : ($this->map->isPubliclyAccessible() ? Permission::Viewer : null);

        return [
            'permission' => $permission?->value,
            'active_character_has_access' => $this->user instanceof User && $this->activeCharacterHasAccess(),
        ];
    }

    /**
     * Check if the active character has any access to the map.
     * This checks the character directly, their corporation, and their alliance.
     */
    private function activeCharacterHasAccess(): bool
    {
        $activeCharacter = $this->user?->active_character;

        if (! $activeCharacter instanceof Character) {
            return false;
        }

        $accessibleIds = array_filter([
            $activeCharacter->id,
            $activeCharacter->corporation_id,
            $activeCharacter->alliance_id,
        ]);

        return $this->map->mapAccessors()
            ->notExpired()
            ->whereIn('accessible_id', $accessibleIds)
            ->exists();
    }
}
