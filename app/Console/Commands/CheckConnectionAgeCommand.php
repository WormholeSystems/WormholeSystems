<?php

declare(strict_types=1);

namespace App\Console\Commands;

use App\Actions\MapConnections\UpdateMapConnectionAction;
use App\Data\MapConnectionData;
use App\Enums\LifetimeStatus;
use App\Models\MapConnection;
use Illuminate\Console\Command;
use Illuminate\Database\Eloquent\Builder;
use Throwable;

use function now;

final class CheckConnectionAgeCommand extends AppCommand
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
    protected $description = 'Check the age of wormhole connections and updates their lifetime status';

    public function __construct(
        private readonly UpdateMapConnectionAction $updateMapConnectionAction
    ) {
        parent::__construct();
    }

    /**
     * Execute the console command.
     *
     * @throws Throwable
     */
    public function handle(): int
    {
        $connections = MapConnection::query()
            ->where(
                fn (Builder $query) => $query
                    ->whereHas('fromMapSolarsystem', fn (Builder $query) => $query->whereHas('wormholeSystem'))
                    ->orWhereHas('toMapSolarsystem', fn (Builder $query) => $query->whereHas('wormholeSystem'))
            )
            ->where('lifetime', '!=', LifetimeStatus::Critical)
            ->cursor()->map($this->calculateLifetimeStatusForConnection(...))->filter();

        $this->info("Updated lifetime status for {$connections->count()} connections.");

        return self::SUCCESS;
    }

    /**
     * @throws Throwable
     */
    private function calculateLifetimeStatusForConnection(MapConnection $connection): bool
    {
        $calculatedLifetimeStatus = $this->calculateLifetimeStatus($connection);

        if (! $this->shouldUpdateLifetime($connection, $calculatedLifetimeStatus)) {
            return false;
        }

        $this->updateMapConnectionAction->handle($connection, MapConnectionData::from([
            'lifetime' => $calculatedLifetimeStatus,
            'lifetime_updated_at' => now(),
        ]));

        return true;
    }

    private function shouldUpdateLifetime(MapConnection $connection, LifetimeStatus $calculatedStatus): bool
    {
        $currentSeverity = $this->getLifetimeSeverity($connection->lifetime);
        $calculatedSeverity = $this->getLifetimeSeverity($calculatedStatus);

        return $calculatedSeverity > $currentSeverity;
    }

    private function getLifetimeSeverity(LifetimeStatus $status): int
    {
        return match ($status) {
            LifetimeStatus::Healthy => 1,
            LifetimeStatus::EndOfLife => 2,
            LifetimeStatus::Critical => 3,
        };
    }

    private function calculateLifetimeStatus(MapConnection $connection): LifetimeStatus
    {
        if ($connection->lifetime === LifetimeStatus::EndOfLife) {
            return $this->calculateLifetimeStatusForEolConnection($connection);
        }

        $createdAt = $connection->connected_at ?? $connection->created_at;
        $hoursAlive = now()->diffInHours($createdAt, absolute: true);

        $isC6Connection = $this->isC6Connection($connection);
        $isDrifterConnection = $this->isDrifterConnection($connection);

        if ($isDrifterConnection) {
            return match (true) {
                $hoursAlive >= 15 => LifetimeStatus::Critical, // <1h remaining
                $hoursAlive >= 12 => LifetimeStatus::EndOfLife, // <4h remaining
                default => LifetimeStatus::Healthy,
            };
        }

        if ($isC6Connection) {
            return match (true) {
                $hoursAlive >= 47 => LifetimeStatus::Critical, // <1h remaining
                $hoursAlive >= 44 => LifetimeStatus::EndOfLife, // <4h remaining
                default => LifetimeStatus::Healthy,
            };
        }

        return match (true) {
            $hoursAlive >= 23 => LifetimeStatus::Critical, // <1h remaining
            $hoursAlive >= 20 => LifetimeStatus::EndOfLife, // <4h remaining
            default => LifetimeStatus::Healthy,
        };
    }

    private function calculateLifetimeStatusForEolConnection(MapConnection $connection): LifetimeStatus
    {
        $marked_as_eol_at = $connection->lifetime_updated_at;
        $time_passed = now()->diffInHours($marked_as_eol_at, absolute: true);

        return match (true) {
            $time_passed >= 3 => LifetimeStatus::Critical, // <1h remaining
            default => LifetimeStatus::EndOfLife, // <4h remaining
        };
    }

    private function isC6Connection(MapConnection $connection): bool
    {
        return $connection->fromMapSolarsystem->wormholeSystem?->class === 6
            && $connection->toMapSolarsystem->wormholeSystem === null
            || $connection->toMapSolarsystem->wormholeSystem?->class === 6
            && $connection->fromMapSolarsystem->wormholeSystem === null;
    }

    private function isDrifterConnection(MapConnection $connection): bool
    {
        $drifterClasses = [14, 15, 16, 17, 18];

        return in_array($connection->fromMapSolarsystem->wormholeSystem?->class, $drifterClasses, true)
            && $connection->toMapSolarsystem->wormholeSystem === null
            || in_array($connection->toMapSolarsystem->wormholeSystem?->class, $drifterClasses, true)
            && $connection->fromMapSolarsystem->wormholeSystem === null;
    }
}
