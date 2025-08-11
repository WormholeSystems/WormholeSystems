<?php

declare(strict_types=1);

namespace Database\Seeders;

use App\Actions\MapConnections\CreateMapConnectionWithIDsAction;
use App\Actions\MapSolarsystem\StoreMapSolarsystemAction;
use App\Models\Map;
use Illuminate\Database\Seeder;

final class MapSeeder extends Seeder
{
    /**
     * Run the database seeds.
     */
    public function run(StoreMapSolarsystemAction $addSolarsystemToMapAction, CreateMapConnectionWithIDsAction $connectMapSolarsystemsWithIDsAction): void
    {
        $map = Map::create([
            'name' => 'Test Map',
        ]);

        $solarsystems = collect([
            31001171,
            31000054,
            31002106,
        ]);

        foreach ($solarsystems as $solarsystemId) {
            $addSolarsystemToMapAction->handle($map, [
                'solarsystem_id' => $solarsystemId,
            ]);
        }

        foreach ($solarsystems as $solarsystemId) {
            $randomSolarsystemId = $solarsystems->filter(fn ($solarsystem): bool => $solarsystem !== $solarsystemId)
                ->random();

            $connectMapSolarsystemsWithIDsAction->handle(
                $map,
                $solarsystemId,
                $randomSolarsystemId,
            );
        }

    }
}
