<?php

declare(strict_types=1);

namespace App\Console\Commands\Database;

use Illuminate\Console\Command;
use Illuminate\Container\Attributes\Config;
use Illuminate\Support\Facades\Process;
use Illuminate\Support\Facades\Storage;

final class CacheDatabase extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'db:cache';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Caches the database for faster seed operations';

    /**
     * Execute the console command.
     */
    public function handle(
        #[Config('database.connections.mysql.host')] string $host,
        #[Config('database.connections.mysql.port')] int $port,
        #[Config('database.connections.mysql.database')] string $database,
        #[Config('database.connections.mysql.username')] string $username,
        #[Config('database.connections.mysql.password')] string $password,
        #[Config('database.cache.path')] string $cachePath,
        #[Config('database.cache.file')] string $cacheFile,
    ): void {
        if (! Storage::exists($cachePath)) {
            Storage::makeDirectory($cachePath);
        }

        $file = sprintf('%s/%s', Storage::path($cachePath), $cacheFile);

        $process = Process::run([
            'mysqldump',
            '--host='.$host,
            '--port='.$port,
            '--user='.$username,
            '--password='.$password,
            '--no-create-info',
            '--no-create-db',
            '--insert-ignore',
            $database,
            '--result-file='.$file,
        ]);

        if ($process->successful()) {
            $this->info($process->output());
            $this->info('Database cached successfully.');
        } else {
            $this->error('Failed to cache the database: '.$process->errorOutput());
        }

    }
}
