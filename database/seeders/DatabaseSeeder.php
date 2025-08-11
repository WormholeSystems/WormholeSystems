<?php

declare(strict_types=1);

namespace Database\Seeders;

// use Illuminate\Database\Console\Seeds\WithoutModelEvents;
use Illuminate\Database\Seeder;
use Illuminate\Support\Facades\Artisan;

final class DatabaseSeeder extends Seeder
{
    /**
     * Seed the application's database.
     */
    public function run(): void
    {
        Artisan::call('sde:seed');
        $this->call(EsiScopeSeeder::class);
        $this->call(WormholeEffectSeeder::class);
        $this->call(WormholeSeeder::class);
        $this->call(WormholeSystemSeeder::class);
        $this->call(MapSeeder::class);
    }
}
