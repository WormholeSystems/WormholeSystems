<?php

declare(strict_types=1);

namespace App\Actions\MapAccess;

use App\Enums\Permission;
use App\Models\Alliance;
use App\Models\Character;
use App\Models\Corporation;
use App\Models\Map;
use App\Models\MapAccess;
use Illuminate\Support\Facades\DB;
use Throwable;

final readonly class CreateMapAccessAction
{
    /**
     * Execute the action.
     *
     * @throws Throwable
     */
    public function handle(Map $map, Character|Corporation|Alliance $accessor, bool $is_owner = false, Permission $permission = Permission::Write): MapAccess
    {
        return DB::transaction(fn (): MapAccess => $accessor->mapAccesses()->create([
            'map_id' => $map->id,
            'is_owner' => $is_owner,
            'permission' => $permission,
            'accessible_id' => $accessor->id,
        ]), 5);
    }
}
