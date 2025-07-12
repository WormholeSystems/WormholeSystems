<?php

namespace App\Console\Commands;

use App\Actions\MapConnections\UpdateMapConnectionAction;
use App\Models\MapConnection;
use Illuminate\Console\Command;
use Illuminate\Database\Eloquent\Builder;

class CheckConnectionAgeCommand extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'app:check-connection-age';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Check the age of wormhole connections ans marks them as eol';

    /**
     * Execute the console command.
     */
    public function handle(UpdateMapConnectionAction $updateMapConnectionAction): int
    {
        $eol_connections = MapConnection::query()
            ->where(
                fn(Builder $query) => $query
                    ->whereHas('fromMapSolarsystem', fn(Builder $query) => $query->whereHas('wormholeSystem'))
                    ->orWhereHas('toMapSolarsystem', fn(Builder $query) => $query->whereHas('wormholeSystem'))
            )
            ->where('created_at', '<=', now()->subHours(20))
            ->where('is_eol', false)
            ->get();

        $eol_connections->each(fn(MapConnection $connection) => $updateMapConnectionAction->handle($connection, [
            'is_eol' => true,
        ]));


        $this->info('Connections older than 20 hours have been marked as EOL.');

        return self::SUCCESS;
    }
}
