<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Http\Controllers\Concerns\HasMaintainerPeriodResolution;
use App\Http\Resources\MapInfoResource;
use App\Models\Map;
use App\Models\User;
use App\Services\Statistics\MaintainerPeriod;
use App\Services\Statistics\MaintainerReportReader;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Gate;
use Inertia\Inertia;
use Inertia\Response;

final class MapStatisticsPageController extends Controller
{
    use HasMaintainerPeriodResolution;

    public function __construct(
        #[CurrentUser] private readonly User $user,
        private readonly MaintainerReportReader $reader,
    ) {}

    /**
     * Show the maintainer leaderboard page.
     *
     * Reads through MaintainerReportReader, same as the API -- a finished period is
     * finalized on demand and read back frozen, never recomputed live (decision #5).
     */
    public function show(Request $request, Map $map): Response
    {
        Gate::authorize('viewLeaderboard', $map);

        $period = $this->resolveMaintainerPeriod($request);

        return Inertia::render('maps/ShowLeaderboard', [
            'map' => $map->toResource(MapInfoResource::class),
            'is_owner' => Gate::allows('delete', $map),
            'permission' => $map->getUserPermission($this->user)?->value,
            'period' => $period->toString(),
            'available_periods' => collect(MaintainerPeriod::selectable())
                ->map(fn (MaintainerPeriod $selectable): array => [
                    'value' => $selectable->toString(),
                    'label' => $selectable->label(),
                    'is_current' => $selectable->isCurrent(),
                ])
                ->all(),
            'entries' => $this->reader->aggregated($map, $period)->values()->all(),
        ]);
    }
}
