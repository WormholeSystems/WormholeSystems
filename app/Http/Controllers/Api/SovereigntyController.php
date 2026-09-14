<?php

declare(strict_types=1);

namespace App\Http\Controllers\Api;

use App\Http\Controllers\Controller;
use App\Http\Resources\SovereigntyResource;
use App\Models\Sovereignty;
use Illuminate\Http\JsonResponse;
use Illuminate\Support\Facades\Cache;

final class SovereigntyController extends Controller
{
    /**
     * Sovereignty for every claimed solar system, keyed by solar system id.
     *
     * The full response covers thousands of systems; the example below is trimmed.
     *
     * @response {
     *   "30000142": {"id": 30000142, "alliance": null, "corporation": null, "faction": {"id": 500001, "name": "Caldari State"}},
     *   "30004759": {"id": 30004759, "alliance": {"id": 99000001, "name": "Example Alliance", "ticker": "EXMPL"}, "corporation": {"id": 98000001, "name": "Example Corporation", "ticker": "EXCRP"}, "faction": null}
     * }
     */
    public function index(): JsonResponse
    {
        $data = Cache::remember('sovereignty', 60 * 60 * 24, function (): array {
            $sovereignties = Sovereignty::query()
                ->with(['alliance', 'corporation', 'faction'])
                ->get();

            return $sovereignties
                ->mapWithKeys(fn (Sovereignty $sovereignty): array => [
                    $sovereignty->solarsystem_id => SovereigntyResource::make($sovereignty)->resolve(),
                ])
                ->all();
        });

        return response()
            ->json($data)
            ->header('Cache-Control', 'public, max-age=86400');
    }
}
