<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\WormholeSystem;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin WormholeSystem
 */
final class ThreatAnalysisResource extends JsonResource
{
    /**
     * @return array<string, mixed>
     */
    public function toArray(Request $request): array
    {
        return [
            'solarsystem_id' => $this->id,
            'threat_level' => $this->threat_level,
            'threat_data' => $this->threat_data ?? [],
            'threat_analyzed_at' => $this->threat_analyzed_at?->toISOString(),
        ];
    }
}
