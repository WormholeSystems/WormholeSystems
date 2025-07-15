<?php

declare(strict_types=1);

namespace App\Actions;

use App\Console\Commands\Killmails\AnalyzeWormholeSystems;
use Illuminate\Support\Facades\Artisan;

final readonly class CreateStatisticsAction
{
    /**
     * Execute the action.
     */
    public function handle(array $data): void
    {
        $args = [
            'map' => $data['map_id'],
        ];

        if (isset($data['active_threshold'])) {
            $args['--active-threshold'] = $data['active_threshold'];
        }

        if (isset($data['days_active'])) {
            $args['--days-active'] = $data['days_active'];
        }

        if (isset($data['days_ago'])) {
            $args['--days-ago'] = $data['days_ago'];
        }

        if (isset($data['top'])) {
            $args['--top'] = $data['top'];
        }

        if (isset($data['hostile_threshold'])) {
            $args['--hostile-threshold'] = $data['hostile_threshold'];
        }

        Artisan::queue(AnalyzeWormholeSystems::class, $args);
    }
}
