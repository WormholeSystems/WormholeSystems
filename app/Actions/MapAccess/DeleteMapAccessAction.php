<?php

declare(strict_types=1);

namespace App\Actions\MapAccess;

use App\Models\MapAccess;
use Illuminate\Support\Facades\DB;
use Throwable;

final readonly class DeleteMapAccessAction
{
    /**
     * Execute the action.
     *
     * @throws Throwable
     */
    public function handle(MapAccess $mapAccess): void
    {
        DB::transaction(function () use ($mapAccess): void {
            $mapAccess->delete();
        });
    }
}
