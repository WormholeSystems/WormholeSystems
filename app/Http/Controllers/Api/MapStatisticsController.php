<?php

declare(strict_types=1);

namespace App\Http\Controllers\Api;

use App\Http\Controllers\Concerns\HasMaintainerPeriodResolution;
use App\Http\Controllers\Controller;
use App\Http\Resources\MaintainerCharacterStatResource;
use App\Http\Resources\MaintainerEntryResource;
use App\Models\Map;
use App\Services\Statistics\MaintainerReportReader;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Gate;

final class MapStatisticsController extends Controller
{
    use HasMaintainerPeriodResolution;

    public function __construct(
        private readonly MaintainerReportReader $reader,
    ) {}

    public function aggregated(Request $request, Map $map): JsonResponse
    {
        Gate::authorize('viewLeaderboard', $map);

        $period = $this->resolveMaintainerPeriod($request);

        return response()->json([
            'data' => $this->reader->aggregated($map, $period)->toResourceCollection(MaintainerEntryResource::class),
        ]);
    }

    public function details(Request $request, Map $map): JsonResponse
    {
        Gate::authorize('viewLeaderboard', $map);

        $period = $this->resolveMaintainerPeriod($request);

        return response()->json([
            'data' => $this->reader->details($map, $period)->toResourceCollection(MaintainerCharacterStatResource::class),
        ]);
    }
}
