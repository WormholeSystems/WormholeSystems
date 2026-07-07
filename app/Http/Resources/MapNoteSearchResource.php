<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\MapSolarsystem;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;
use Illuminate\Support\Str;

/**
 * @mixin MapSolarsystem
 */
final class MapNoteSearchResource extends JsonResource
{
    private const int EXCERPT_RADIUS = 40;

    /**
     * @return array<string, mixed>
     */
    public function toArray(Request $request): array
    {
        return [
            'map_solarsystem_id' => $this->id,
            'solarsystem_id' => $this->solarsystem_id,
            'alias' => $this->alias,
            'occupier_alias' => $this->details->occupier_alias,
            'note_excerpt' => $this->buildExcerpt((string) $this->details->notes, mb_trim((string) $request->string('q'))),
        ];
    }

    private function buildExcerpt(string $notes, string $needle): string
    {
        $flattened = mb_trim((string) preg_replace('/\s+/', ' ', $notes));

        return Str::excerpt($flattened, $needle, ['radius' => self::EXCERPT_RADIUS, 'omission' => '…'])
            ?? mb_substr($flattened, 0, self::EXCERPT_RADIUS * 2);
    }
}
