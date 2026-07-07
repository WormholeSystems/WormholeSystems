<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Enums\Permission;
use App\Http\Requests\MapSearchRequest;
use App\Http\Resources\MapNoteSearchResource;
use App\Http\Resources\MapOccupierSearchResource;
use App\Http\Resources\MapThreatSearchResource;
use App\Models\Map;
use App\Models\MapSolarsystemDetails;
use App\Models\User;
use App\Models\WormholeSystemThreat;
use Illuminate\Http\JsonResponse;
use Illuminate\Support\Collection;
use Illuminate\Support\Facades\Gate;

/**
 * The command palette's server-side search. Ranking happens ahead of every limit so
 * a good match can never be cut; notes are omitted for viewers.
 */
final class MapSearchController extends Controller
{
    private const int ENTITY_LIMIT = 10;

    private const int NOTE_LIMIT = 10;

    private const int OCCUPIER_LIMIT = 10;

    public function index(MapSearchRequest $request, Map $map): JsonResponse
    {
        Gate::authorize('view', $map);

        $needle = mb_strtolower(mb_trim((string) $request->string('q')));
        $escaped = addcslashes($needle, '%_\\');
        $contains = "%{$escaped}%";
        $prefix = "{$escaped}%";

        return response()->json([
            'threats' => MapThreatSearchResource::collection($this->threats($map, $needle, $contains, $prefix)),
            'notes' => MapNoteSearchResource::collection($this->notes($request, $map, $needle, $contains)),
            'occupiers' => MapOccupierSearchResource::collection($this->occupiers($map, $contains, $prefix)),
        ]);
    }

    /**
     * EVE ids are unique across entity types, so entity_id alone identifies an organisation.
     *
     * @return Collection<int, Collection<int, WormholeSystemThreat>>
     */
    private function threats(Map $map, string $needle, string $contains, string $prefix): Collection
    {
        $top_entities = WormholeSystemThreat::query()
            ->select('entity_id')
            ->selectRaw('CASE WHEN name = ? THEN 0 WHEN name LIKE ? THEN 1 ELSE 2 END AS name_rank', [$needle, $prefix])
            ->selectRaw('SUM(kills) AS total_kills')
            ->whereLike('name', $contains)
            ->groupBy('entity_id', 'entity_type', 'name')
            ->orderBy('name_rank')
            ->orderByDesc('total_kills')
            ->limit(self::ENTITY_LIMIT);

        return WormholeSystemThreat::query()
            ->select('wormhole_system_threats.*')
            ->addSelect([
                'occupier_alias' => MapSolarsystemDetails::query()
                    ->select('occupier_alias')
                    ->where('map_id', $map->id)
                    ->whereColumn('solarsystem_id', 'wormhole_system_threats.wormhole_system_id'),
            ])
            ->joinSub($top_entities, 'top_entities', 'top_entities.entity_id', '=', 'wormhole_system_threats.entity_id')
            ->orderBy('top_entities.name_rank')
            ->orderByDesc('top_entities.total_kills')
            ->get()
            ->groupBy('entity_id')
            ->values();
    }

    private function notes(MapSearchRequest $request, Map $map, string $needle, string $contains): Collection
    {
        $user = $request->user();

        if (! $user instanceof User || $map->getUserPermission($user) === Permission::Viewer) {
            return new Collection;
        }

        return $map->mapSolarsystems()
            ->join('map_solarsystem_details', 'map_solarsystem_details.id', '=', 'map_solarsystems.map_solarsystem_details_id')
            ->whereLike('map_solarsystem_details.notes', $contains)
            ->orderByRaw('INSTR(LOWER(map_solarsystem_details.notes), ?)', [$needle])
            ->limit(self::NOTE_LIMIT)
            ->with('details')
            ->get(['map_solarsystems.*'])
            ->toBase();
    }

    /** Details rows survive system removal, so this also finds systems no longer on the map. */
    private function occupiers(Map $map, string $contains, string $prefix): Collection
    {
        return MapSolarsystemDetails::query()
            ->where('map_id', $map->id)
            ->whereLike('occupier_alias', $contains)
            ->orderByRaw('CASE WHEN occupier_alias LIKE ? THEN 0 ELSE 1 END, occupier_alias', [$prefix])
            ->limit(self::OCCUPIER_LIMIT)
            ->get()
            ->toBase();
    }
}
