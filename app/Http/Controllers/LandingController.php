<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Http\Resources\KillmailResource;
use App\Models\Killmail;
use Illuminate\Http\Resources\Json\ResourceCollection;
use Inertia\Inertia;
use Inertia\Response;

final class LandingController extends Controller
{
    public function index(): Response
    {
        return Inertia::render('Landing', [
            'killmails' => fn (): ResourceCollection => $this->getLatestKillmails(),
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
            ->whereRelation('solarsystem', 'type', 'wormhole')
            ->orderByDesc('id')
            ->limit(12)
            ->get()
            ->toResourceCollection(KillmailResource::class);
    }
}
