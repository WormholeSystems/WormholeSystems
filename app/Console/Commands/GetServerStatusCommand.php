<?php

declare(strict_types=1);

namespace App\Console\Commands;

use App\Events\ServerStatusUpdatedEvent;
use App\Models\ServerStatus;
use Illuminate\Console\Command;
use NicolasKion\Esi\DTO\Status;
use NicolasKion\Esi\Esi;
use Throwable;

use function sprintf;

final class GetServerStatusCommand extends AppCommand
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'app:get-server-status';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Command description';

    /**
     * Execute the console command.
     */
    public function handle(Esi $esi): int
    {
        try {
            $response = $esi->getStatus();
        } catch (Throwable $e) {
            $this->error(sprintf(
                'Encountered an error while fetching server status, server might be down: %s',
                $e->getMessage()
            ));
            $this->markServerAsDown();

            return self::SUCCESS;
        }

        if ($response->failed()) {
            $this->error(sprintf(
                'Failed to fetch server status, server might be down. Status code: %d, body: %s',
                $response->error->code,
                $response->error->body ?? ''
            ));
            $this->markServerAsDown();

            return self::SUCCESS;
        }

        $status = $this->updateServerStatus($response->data);

        $this->info(sprintf(
            'Server status updated: %d players, version %s, started at %s, VIP: %s',
            $status->players,
            $status->server_version,
            $status->start_time,
            $status->vip ? 'Yes' : 'No'
        ));

        return self::SUCCESS;
    }

    private function markServerAsDown(): void
    {
        $status = ServerStatus::query()->create([
            'players' => 0,
            'server_version' => 'unknown',
            'start_time' => now(),
            'vip' => false,
        ]);

        $this->notifySubscribers($status);
    }

    private function updateServerStatus(Status $status): ServerStatus
    {
        $serverStatus = ServerStatus::query()->create([
            'players' => $status->players,
            'server_version' => $status->server_version,
            'start_time' => $status->start_time,
            'vip' => $status->vip,
        ]);

        $this->notifySubscribers($serverStatus);

        return $serverStatus;
    }

    private function notifySubscribers(ServerStatus $status): void
    {
        broadcast(new ServerStatusUpdatedEvent($status));
    }
}
