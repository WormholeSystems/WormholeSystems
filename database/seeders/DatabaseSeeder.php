<?php

namespace Database\Seeders;

// use Illuminate\Database\Console\Seeds\WithoutModelEvents;
use Illuminate\Container\Attributes\Config;
use Illuminate\Database\Seeder;
use Illuminate\Support\Facades\Artisan;

class DatabaseSeeder extends Seeder
{
    /**
     * Seed the application's database.
     */
    public function run(
        #[Config('database.cache.file')] string $cacheFile,
        #[Config('database.cache.path')] string $cachePath,
    ): void {

        if (file_exists(sprintf('%s/%s', $cachePath, $cacheFile))) {
            $this->call('db:restore', [
                '--database' => 'test_database',
            ]);
        }

        Artisan::call('sde:seed');
        $this->call(WormholeEffectSeeder::class);
        $this->call(WormholeSeeder::class);
        $this->call(WormholeSystemSeeder::class);
        $this->call(MapSeeder::class);
    }
}
