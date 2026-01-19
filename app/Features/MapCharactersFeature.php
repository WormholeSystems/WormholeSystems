<?php

declare(strict_types=1);

namespace App\Features;

use App\Http\Resources\CharacterResource;
use App\Models\Character;
use App\Models\Map;
use App\Scopes\CharacterHasMapAccess;
use App\Scopes\CharacterIsOnline;
use App\Scopes\UserAllowedMapTracking;
use Illuminate\Http\Resources\Json\ResourceCollection;
use Inertia\ProvidesInertiaProperties;
use Inertia\RenderContext;
use Throwable;

final readonly class MapCharactersFeature implements ProvidesInertiaProperties
{
    public function __construct(private Map $map, private bool $can_view_characters) {}

    public function toInertiaProperties(RenderContext $context): array
    {
        return [
            'map_characters' => fn (): ?ResourceCollection => $this->can_view_characters ? $this->getMapCharacters() : null,
        ];
    }

    /**
     * @throws Throwable
     */
    private function getMapCharacters(): ResourceCollection
    {
        return Character::query()
            ->hasTokenWithTrackingScopes()
            ->with([
                'characterStatus',
                'corporation',
                'alliance',
                'faction',
            ])
            ->tap(new UserAllowedMapTracking($this->map))
            ->tap(new CharacterHasMapAccess($this->map, without_guests: true))
            ->tap(new CharacterIsOnline)
            ->get()
            ->toResourceCollection(CharacterResource::class);
    }
}
