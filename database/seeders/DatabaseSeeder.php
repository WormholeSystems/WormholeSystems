<?php

declare(strict_types=1);

namespace Database\Seeders;

// use Illuminate\Database\Console\Seeds\WithoutModelEvents;
use Illuminate\Database\Seeder;
use NicolasKion\SDE\Commands\SeedCommand;

final class DatabaseSeeder extends Seeder
{
    /**
     * Seed the application's database.
     */
    public function run(): void
    {
        $this->command->call(SeedCommand::class);
        $this->call(EsiScopeSeeder::class);
        $this->call(WormholeEffectSeeder::class);
        $this->call(WormholeSeeder::class);
        $this->call(WormholeSystemSeeder::class);
        $this->call(MapSeeder::class);
    }
}
