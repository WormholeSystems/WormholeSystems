<?php

declare(strict_types=1);

namespace App\Actions\MapAccess;

use Illuminate\Support\Facades\DB;

final readonly class DeleteMapAccessAction
{
    /**
     * Execute the action.
     */
    public function handle(): void
    {
        DB::transaction(function (): void {
            //
        });
    }
}
