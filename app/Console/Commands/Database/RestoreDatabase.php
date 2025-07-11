<?php

namespace App\Console\Commands\Database;

use Illuminate\Console\Command;
use Illuminate\Container\Attributes\Config;
use Illuminate\Support\Facades\Process;
use Illuminate\Support\Facades\Storage;

class RestoreDatabase extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'db:restore';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Restores the database from a cached file';

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
    ) {
        if (! Storage::exists($cachePath)) {
            $this->error('Cache directory does not exist.');

            return;
        }

        $file = sprintf('%s/%s', Storage::path($cachePath), $cacheFile);

        if (! file_exists($file)) {
            $this->error('Cache file does not exist.');

            return;
        }

        $process = Process::run([
            'mysql',
            '--host='.$host,
            '--port='.$port,
            '--user='.$username,
            '--password='.$password,
            $database,
            '--default-character-set=utf8mb4',
            '--init-command=SET FOREIGN_KEY_CHECKS=0;',
            '--execute=source '.$file,
        ]);

        if ($process->successful()) {
            $this->info($process->output());
            $this->info('Database restored successfully.');
        } else {
            $this->error('Failed to restore the database: '.$process->errorOutput());
        }

    }
}
