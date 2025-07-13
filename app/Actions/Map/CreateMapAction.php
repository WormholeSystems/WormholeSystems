<?php

namespace App\Actions\Map;

use App\Actions\MapAccess\CreateMapAccessAction;
use App\Actions\MapRouteSolarsystems\CreateMapRouteSolarsystemAction;
use App\Models\Character;
use App\Models\Map;
use App\Models\Solarsystem;
use Illuminate\Support\Facades\DB;
use Throwable;

readonly class CreateMapAction
{
    /**
     * Create a new class instance.
     */
    public function __construct(
        private CreateMapAccessAction $createMapAccessAction,
        private CreateMapRouteSolarsystemAction $createMapRouteSolarsystemAction,
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

            $default_systems = Solarsystem::query()->whereIn('name', [
                'Jita',
                'Amarr',
                'Dodixie',
                'Rens',
                'Hek',
            ])->get('id');

            foreach ($default_systems as $system) {
                $this->createMapRouteSolarsystemAction->handle([
                    'map_id' => $map->id,
                    'solarsystem_id' => $system->id,
                    'is_pinned' => true,
                ]);
            }

            return $map;
        }, 5);
    }
}
