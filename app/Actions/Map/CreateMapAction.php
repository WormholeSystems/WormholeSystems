<?php

namespace App\Actions\Map;

use App\Actions\MapAccess\CreateMapAccessAction;
use App\Models\Character;
use App\Models\Map;
use Illuminate\Support\Facades\DB;
use Throwable;

readonly class CreateMapAction
{
    /**
     * Create a new class instance.
     */
    public function __construct(
        private CreateMapAccessAction $createMapAccessAction,
    ) {
        //
    }

    /**
     * @throws Throwable
     */
    public function handle(Character $character, array $data): Map
    {
        return DB::transaction(function () use ($character, $data): Map {
            $map = Map::query()->create([
                'name' => $data['name'],
            ]);

            $this->createMapAccessAction->handle(
                map: $map,
                accessor: $character,
                is_owner: true,
            );

            return $map;
        }, 5);
    }
}
