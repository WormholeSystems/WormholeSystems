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
    public function index(): JsonResponse
    {
        $data = Cache::remember('sovereignty', 60 * 60 * 24, function (): array {
            $sovereignties = Sovereignty::query()
                ->with(['alliance', 'corporation', 'faction'])
                ->get();

            return $sovereignties
                ->mapWithKeys(fn (Sovereignty $sovereignty) => [
                    $sovereignty->solarsystem_id => SovereigntyResource::make($sovereignty)->resolve(),
                ])
                ->all();
        });

        return response()
            ->json($data)
            ->header('Cache-Control', 'public, max-age=86400');
    }
}
