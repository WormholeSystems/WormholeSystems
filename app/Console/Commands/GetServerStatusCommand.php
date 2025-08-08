<?php

namespace App\Console\Commands;

use App\Events\ServerStatusUpdatedEvent;
use App\Models\ServerStatus;
use Exception;
use Illuminate\Console\Command;
use NicolasKion\Esi\DTO\Status;
use NicolasKion\Esi\Esi;

class GetServerStatusCommand extends Command
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
        } catch (Exception $e) {
            $this->error('Failed to retrieve server status: '.$e->getMessage());

            return self::FAILURE;
        }

        /** @var Status $data */
        $data = $response->data;

        $status = ServerStatus::query()->create([
            'players' => $data->players,
            'server_version' => $data->server_version,
            'start_time' => $data->start_time,
            'vip' => $data->vip,
        ]);

        broadcast(new ServerStatusUpdatedEvent($status));

        $this->info(sprintf(
            'Server status updated: %d players, version %s, started at %s, VIP: %s',
            $data->players,
            $data->server_version,
            $data->start_time,
            $data->vip ? 'Yes' : 'No'
        ));

        return self::SUCCESS;
    }
}
