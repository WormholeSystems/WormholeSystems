<?php

namespace App\Actions\MapSelection;

use App\Models\MapSolarsystem;
use Illuminate\Support\Facades\DB;
use Throwable;

class UpdateMapSelectionAction
{
    /**
     * @throws Throwable
     */
    public function handle(array $data): void
    {
        DB::transaction(function () use ($data) {
            collect($data['map_solarsystems'])
                ->each(function ($item) {
                    MapSolarsystem::where('id', $item['id'])
                        ->update(['position_x' => $item['position_x'], 'position_y' => $item['position_y']]);
                });
        });
    }
}
