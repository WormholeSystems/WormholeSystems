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
                fn (Builder $query) => $query
                    ->whereHas('fromMapSolarsystem', fn (Builder $query) => $query->whereHas('wormholeSystem'))
                    ->orWhereHas('toMapSolarsystem', fn (Builder $query) => $query->whereHas('wormholeSystem'))
            )
            ->whereNot(fn (Builder $query) => $query
                ->where(fn (Builder $query) => $query
                    ->whereHas('fromMapSolarsystem', fn (Builder $query) => $query->whereHas('wormholeSystem', fn (Builder $query) => $query->where('class', 6)))
                    ->whereHas('toMapSolarsystem', fn (Builder $query) => $query->whereDoesntHave('wormholeSystem'))
                )
                ->orWhere(fn (Builder $query) => $query
                    ->whereHas('toMapSolarsystem', fn (Builder $query) => $query->whereHas('wormholeSystem', fn (Builder $query) => $query->where('class', 6)))
                    ->whereHas('fromMapSolarsystem', fn (Builder $query) => $query->whereDoesntHave('wormholeSystem'))
                ))
            ->where('created_at', '<=', now()->subHours(20))
            ->where('is_eol', false)
            ->get();

        $eol_connections->each(fn (MapConnection $connection): \App\Models\MapConnection => $updateMapConnectionAction->handle($connection, [
            'is_eol' => true,
        ]));

        $c6_connections = MapConnection::query()
            ->where(
                fn (Builder $query) => $query
                    ->where(fn (Builder $query) => $query
                        ->whereHas('fromMapSolarsystem', fn (Builder $query) => $query->whereHas('wormholeSystem', fn (Builder $query) => $query->where('class', 6)))
                        ->whereHas('toMapSolarsystem', fn (Builder $query) => $query->whereDoesntHave('wormholeSystem'))
                    )
                    ->orWhere(fn (Builder $query) => $query
                        ->whereHas('toMapSolarsystem', fn (Builder $query) => $query->whereHas('wormholeSystem', fn (Builder $query) => $query->where('class', 6)))
                        ->whereHas('fromMapSolarsystem', fn (Builder $query) => $query->whereDoesntHave('wormholeSystem'))
                    ))
            ->where('created_at', '<=', now()->subHours(44))
            ->where('is_eol', false)
            ->get();

        $c6_connections->each(fn (MapConnection $connection): \App\Models\MapConnection => $updateMapConnectionAction->handle($connection, [
            'is_eol' => true,
        ]));

        $this->info('Old connections marked as EOL: '.($eol_connections->count() + $c6_connections->count()));

        return self::SUCCESS;
    }
}
