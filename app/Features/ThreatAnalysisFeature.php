<?php

declare(strict_types=1);

namespace App\Features;

use App\Enums\RemovableCard;
use App\Http\Resources\ThreatAnalysisResource;
use App\Models\MapSolarsystem;
use App\Models\WormholeSystem;
use Inertia\Inertia;
use Inertia\ProvidesInertiaProperties;
use Inertia\RenderContext;

final readonly class ThreatAnalysisFeature implements ProvidesInertiaProperties
{
    /**
     * @param  string[]  $hiddenCards
     */
    public function __construct(
        private ?MapSolarsystem $selectedSolarsystem,
        private array $hiddenCards = [],
    ) {}

    public function toInertiaProperties(RenderContext $context): array
    {
        if (in_array(RemovableCard::ThreatAnalysis->value, $this->hiddenCards)) {
            return [];
        }

        return [
            'threat_analysis' => Inertia::defer($this->getThreatAnalysis(...)),
        ];
    }

    private function getThreatAnalysis(): ?ThreatAnalysisResource
    {
        if (! $this->selectedSolarsystem instanceof MapSolarsystem) {
            return null;
        }

        $wormholeSystem = WormholeSystem::query()
            ->where('id', $this->selectedSolarsystem->solarsystem_id)
            ->first();

        if (! $wormholeSystem instanceof WormholeSystem) {
            return null;
        }

        return new ThreatAnalysisResource($wormholeSystem);
    }
}
