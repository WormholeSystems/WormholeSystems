<?php

declare(strict_types=1);

namespace Database\Seeders;

// use Illuminate\Database\Console\Seeds\WithoutModelEvents;
use Illuminate\Database\Seeder;
use Illuminate\Support\Facades\Artisan;

final class RestoredSeeder extends Seeder
{
    /**
     * Seed the application's database.
     */
    public function run(): void
    {

        Artisan::call('db:restore', [
            '--force' => true,
            '--database' => 'test_database',
        ]);
    }
}
