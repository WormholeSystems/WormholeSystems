<?php

declare(strict_types=1);

namespace App\Features;

use App\Enums\Permission;
use App\Models\Map;
use App\Models\User;
use Illuminate\Support\Facades\Gate;
use Inertia\ProvidesInertiaProperties;
use Inertia\RenderContext;

final readonly class MapPermissionsFeature implements ProvidesInertiaProperties
{
    public function __construct(
        private Map $map,
        private User $user,
    ) {}

    public function toInertiaProperties(RenderContext $context): array
    {
        return [
            'has_write_access' => Gate::allows('update', $this->map),
            'has_guest_access' => $this->map->getUserPermission($this->user) === Permission::Guest,
        ];
    }
}
