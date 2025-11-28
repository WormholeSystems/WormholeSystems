<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Http\Resources\RegionResource;
use App\Http\Resources\UniverseSolarsystemResource;
use App\Models\Region;
use App\Models\Solarsystem;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Inertia\Inertia;
use Inertia\Response;

final class UniverseMapController extends Controller
{
    /**
     * Normalization factor - must match UniverseSolarsystemResource.
     */
    private const float COORDINATE_SCALE = 1e14;

    public function show(Request $request): Response
    {
        return Inertia::render('universe-map/ShowUniverseMap', [
            'solarsystems' => $this->getSolarsystems(...),
            'connections' => $this->getConnections(...),
            'regions' => $this->getRegions(...),
            'bounds' => $this->getBounds(...),
            'selectedSystemDetails' => Inertia::lazy(fn (): ?array => $this->getSelectedSystemDetails($request)),
        ]);
    }

    private function getSolarsystems(): mixed
    {
        return Solarsystem::query()
            ->whereNotNull('pos_2d_x')
            ->whereNotNull('pos_2d_y')
            ->with(['region', 'constellation', 'sovereignty.alliance', 'sovereignty.corporation', 'sovereignty.faction'])
            ->withCount(['stations'])
            ->withCount(['celestials as belts_count' => fn ($q) => $q->whereIn('group_id', [9, 4430, 4930, 4935])])
            ->get()
            ->toResourceCollection(UniverseSolarsystemResource::class);
    }

    private function getConnections(): mixed
    {
        $solarsystemIds = Solarsystem::query()
            ->whereNotNull('pos_2d_x')
            ->whereNotNull('pos_2d_y')
            ->pluck('id');

        return DB::table('solarsystem_connections')
            ->whereIn('from_solarsystem_id', $solarsystemIds)
            ->whereIn('to_solarsystem_id', $solarsystemIds)
            ->select(['from_solarsystem_id', 'to_solarsystem_id', 'is_regional'])
            ->get()
            ->map(fn ($conn): array => [
                'from' => $conn->from_solarsystem_id,
                'to' => $conn->to_solarsystem_id,
                'regional' => (bool) $conn->is_regional,
            ])
            ->values();
    }

    private function getRegions(): mixed
    {
        return Region::query()
            ->whereHas('solarsystems', fn ($query) => $query->whereNotNull('pos_2d_x'))
            ->orderBy('name')
            ->get()
            ->toResourceCollection(RegionResource::class);
    }

    private function getBounds(): array
    {
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

        return [
            'minX' => $bounds->min_x / self::COORDINATE_SCALE,
            'maxX' => $bounds->max_x / self::COORDINATE_SCALE,
            'minY' => $bounds->min_y / self::COORDINATE_SCALE,
            'maxY' => $bounds->max_y / self::COORDINATE_SCALE,
        ];
    }

    private function getSelectedSystemDetails(Request $request): ?array
    {
        $systemId = $request->query('system');
        if (! $systemId) {
            return null;
        }

        $system = Solarsystem::with([
            'region',
            'constellation',
            'sovereignty.alliance',
            'sovereignty.corporation',
            'sovereignty.faction',
            'wormholeSystem',
            'celestials.type',
            'celestials.moons',
            'stations.type',
            'killmails' => fn ($q) => $q->with('shipType')->orderBy('time', 'desc')->limit(10),
        ])->find($systemId);

        if (! $system) {
            return null;
        }

        // Get adjacent systems via the relationship
        $adjacentSystems = $system->adjacentSystems;

        // Group celestials by type
        $planets = $system->celestials->filter(fn ($c): bool => $c->group_id === 7);
        $belts = $system->celestials->filter(fn ($c): bool => in_array($c->group_id, [9, 4430, 4930, 4935]));
        $stargates = $system->celestials->filter(fn ($c): bool => $c->group_id === 10);
        $moonsCount = $system->celestials->filter(fn ($c): bool => $c->group_id === 8)->count();

        return [
            'id' => $system->id,
            'name' => $system->name,
            'security' => $system->security,
            'type' => $system->type,
            'region' => [
                'id' => $system->region->id,
                'name' => $system->region->name,
            ],
            'constellation' => [
                'id' => $system->constellation->id,
                'name' => $system->constellation->name,
            ],
            'sovereignty' => $system->sovereignty ? [
                'alliance' => $system->sovereignty->alliance ? [
                    'id' => $system->sovereignty->alliance->id,
                    'name' => $system->sovereignty->alliance->name,
                    'ticker' => $system->sovereignty->alliance->ticker,
                ] : null,
                'corporation' => $system->sovereignty->corporation ? [
                    'id' => $system->sovereignty->corporation->id,
                    'name' => $system->sovereignty->corporation->name,
                    'ticker' => $system->sovereignty->corporation->ticker,
                ] : null,
                'faction' => $system->sovereignty->faction ? [
                    'id' => $system->sovereignty->faction->id,
                    'name' => $system->sovereignty->faction->name,
                ] : null,
            ] : null,
            'wormhole_class' => $system->wormholeSystem?->class,
            'adjacent_systems' => $adjacentSystems->map(fn ($s): array => [
                'id' => $s->id,
                'name' => $s->name,
                'security' => $s->security,
                'region_id' => $s->region_id,
                'region_name' => $s->region->name,
            ])->values(),
            'planets' => $planets->map(fn ($p): array => [
                'id' => $p->id,
                'name' => $p->name,
                'type' => $p->type?->name,
                'type_id' => $p->type_id,
                'moons' => $p->moons->map(fn ($m): array => [
                    'id' => $m->id,
                    'name' => $m->name,
                ])->values(),
            ])->values(),
            'moons_count' => $moonsCount,
            'belts' => $belts->map(fn ($b): array => [
                'id' => $b->id,
                'name' => $b->name,
            ])->values(),
            'stargates' => $stargates->map(fn ($s): array => [
                'id' => $s->id,
                'name' => $s->name,
            ])->values(),
            'stations' => $system->stations->map(fn ($s): array => [
                'id' => $s->id,
                'name' => $s->name,
                'type' => $s->type?->name,
            ])->values(),
            'killmails' => $system->killmails->map(fn ($k): array => [
                'id' => $k->id,
                'time' => $k->time->toISOString(),
                'ship_type' => $k->shipType?->name,
                'ship_type_id' => $k->data->victim->ship_type_id ?? null,
                'zkb' => [
                    'totalValue' => $k->zkb['totalValue'] ?? 0,
                ],
            ])->values(),
        ];
    }
}
