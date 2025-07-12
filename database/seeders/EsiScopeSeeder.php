<?php

namespace Database\Seeders;

use Illuminate\Database\Seeder;
use NicolasKion\Esi\Enums\EsiScope;

class EsiScopeSeeder extends Seeder
{
    /**
     * Run the database seeds.
     */
    public function run(): void
    {
        $defaultScopes = [
            EsiScope::ReadLocations,
            EsiScope::ReadShip,
            EsiScope::ReadOnlineStatus,
            EsiScope::WriteWaypoint,
        ];
        foreach (EsiScope::cases() as $scope) {
            \App\Models\EsiScope::query()->updateOrCreate(
                ['name' => $scope->value],
                ['is_default' => in_array($scope, $defaultScopes)]
            );
        }
    }
}
