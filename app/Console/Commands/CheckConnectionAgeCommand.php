<?php

declare(strict_types=1);

namespace App\Console\Commands;

use App\Actions\MapConnections\UpdateMapConnectionAction;
use App\Data\MapConnectionData;
use App\Models\MapConnection;
use Illuminate\Console\Command;
use Illuminate\Database\Eloquent\Builder;
use Throwable;

final class CheckConnectionAgeCommand extends Command
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
     *
     * @throws Throwable
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
            ->whereNull('marked_as_eol_at')
            ->get();

        $eol_connections->each(fn (MapConnection $connection): MapConnection => $updateMapConnectionAction->handle($connection, MapConnectionData::from([
            'marked_as_eol_at' => now(),
        ])));

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
            ->whereNull('marked_as_eol_at')
            ->get();

        $c6_connections->each(fn (MapConnection $connection): MapConnection => $updateMapConnectionAction->handle($connection, MapConnectionData::from([
            'marked_as_eol_at' => now(),
        ])));

        $drifter_connections = MapConnection::query()
            ->where(fn (Builder $query) => $query
                ->where(fn (Builder $query) => $query
                    ->whereHas('fromMapSolarsystem', fn (Builder $query) => $query->whereHas('wormholeSystem', fn (Builder $query) => $query->whereIn('class', [14, 15, 16, 17, 18])))
                    ->whereHas('toMapSolarsystem', fn (Builder $query) => $query->whereDoesntHave('wormholeSystem'))
                )
                ->orWhere(fn (Builder $query) => $query
                    ->whereHas('toMapSolarsystem', fn (Builder $query) => $query->whereHas('wormholeSystem', fn (Builder $query) => $query->whereIn('class', [14, 15, 16, 17, 18])))
                    ->whereHas('fromMapSolarsystem', fn (Builder $query) => $query->whereDoesntHave('wormholeSystem'))
                ))
            ->where('created_at', '<=', now()->subHours(12))
            ->whereNull('marked_as_eol_at')
            ->get();

        $drifter_connections->each(fn (MapConnection $connection): MapConnection => $updateMapConnectionAction->handle($connection, MapConnectionData::from([
            'marked_as_eol_at' => now(),
        ])));

        $this->info('Old connections marked as EOL: '.($eol_connections->count() + $c6_connections->count() + $drifter_connections->count()));

        return self::SUCCESS;
    }
}
