<?php

declare(strict_types=1);

namespace App\Actions\MapAccess;

use App\Enums\Permission;
use App\Models\MapAccess;
use Illuminate\Support\Facades\DB;
use Throwable;

final readonly class UpdateMapAccessAction
{
    /**
     * Execute the action.
     *
     * @throws Throwable
     */
    public function handle(MapAccess $mapAccess, ?bool $is_owner = null, ?Permission $permission = null): void
    {
        DB::transaction(function () use ($mapAccess, $is_owner, $permission): void {
            $mapAccess->update([
                'is_owner' => $is_owner ?? $mapAccess->is_owner,
                'permission' => $permission ?? $mapAccess->permission,
            ]);
        });
    }
}
