<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Http\Resources\RegionResource;
use App\Http\Resources\UniverseSolarsystemResource;
use App\Models\Region;
use App\Models\Solarsystem;
use Illuminate\Support\Facades\DB;
use Inertia\Inertia;
use Inertia\Response;

final class UniverseMapController extends Controller
{
    /**
     * Normalization factor - must match UniverseSolarsystemResource.
     */
    private const float COORDINATE_SCALE = 1e14;

    public function show(): Response
    {
        $solarsystems = Solarsystem::query()
            ->whereNotNull('pos_2d_x')
            ->whereNotNull('pos_2d_y')
            ->with(['region', 'constellation', 'wormholeSystem', 'sovereignty.alliance', 'sovereignty.corporation', 'sovereignty.faction'])
            ->withCount(['stations'])
            ->withCount(['celestials as belts_count' => fn ($q) => $q->whereIn('group_id', [9, 4430, 4930, 4935])])
            ->get();

        // Get bounds for all systems
        $bounds = Solarsystem::query()
            ->whereNotNull('pos_2d_x')
            ->whereNotNull('pos_2d_y')
            ->select([
                DB::raw('MIN(pos_2d_x) as min_x'),
                DB::raw('MAX(pos_2d_x) as max_x'),
                DB::raw('MIN(pos_2d_y) as min_y'),
                DB::raw('MAX(pos_2d_y) as max_y'),
            ])
            ->first();

        // Get stargate connections
        $solarsystemIds = $solarsystems->pluck('id')->toArray();

        $connections = DB::table('solarsystem_connections')
            ->whereIn('from_solarsystem_id', $solarsystemIds)
            ->whereIn('to_solarsystem_id', $solarsystemIds)
            ->select(['from_solarsystem_id', 'to_solarsystem_id', 'is_regional'])
            ->get()
            ->map(fn ($conn): array => [
                'from' => $conn->from_solarsystem_id,
                'to' => $conn->to_solarsystem_id,
                'regional' => (bool) $conn->is_regional,
            ])
            ->values()
            ->toArray();

        $regions = Region::query()
            ->whereHas('solarsystems', fn ($query) => $query->whereNotNull('pos_2d_x'))
            ->orderBy('name')
            ->get()
            ->toResourceCollection(RegionResource::class);

        return Inertia::render('universe-map/ShowUniverseMap', [
            'solarsystems' => $solarsystems->toResourceCollection(UniverseSolarsystemResource::class),
            'connections' => $connections,
            'regions' => $regions,
            'bounds' => [
                'minX' => $bounds->min_x / self::COORDINATE_SCALE,
                'maxX' => $bounds->max_x / self::COORDINATE_SCALE,
                'minY' => $bounds->min_y / self::COORDINATE_SCALE,
                'maxY' => $bounds->max_y / self::COORDINATE_SCALE,
            ],
        ]);
    }
}
