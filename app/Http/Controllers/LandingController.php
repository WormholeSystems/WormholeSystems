<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Http\Resources\KillmailResource;
use App\Models\Killmail;
use App\Models\Solarsystem;
use Illuminate\Http\Resources\Json\ResourceCollection;
use Illuminate\Support\Facades\Cache;
use Inertia\Inertia;
use Inertia\Response;

final class LandingController extends Controller
{
    public function index(): Response
    {
        return Inertia::render('Landing', [
            'killmails' => $this->getLatestKillmails(...),
        ]);
    }

    /**
     * The most recent J-space kills, polled live by the landing page.
     */
    private function getLatestKillmails(): ResourceCollection
    {
        return Killmail::query()
            ->with([
                'shipType',
                'victimCorporation:id,name,ticker',
                'victimAlliance:id,name,ticker',
            ])
            ->whereIn('solarsystem_id', $this->wormholeSolarsystemIds())
            ->orderByDesc('id')
            ->limit(12)
            ->get()
            ->toResourceCollection(KillmailResource::class);
    }

    /**
     * Filtering through the relation makes MySQL sort every killmail in every
     * wormhole system before taking twelve. Passing the ids in keeps it on the
     * primary key, which turns five seconds into twelve milliseconds.
     *
     * @return list<int>
     */
    private function wormholeSolarsystemIds(): array
    {
        return Cache::remember(
            'landing:wormhole-solarsystem-ids',
            60 * 60 * 24,
            fn (): array => Solarsystem::query()->where('type', 'wh')->pluck('id')->all(),
        );
    }
}
