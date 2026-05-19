<?php

declare(strict_types=1);

namespace App\Features;

use App\Enums\RemovableCard;
use App\Http\Resources\RaidableSkyhookResource;
use App\Models\RaidableSkyhook;
use Illuminate\Http\Resources\Json\ResourceCollection;
use Inertia\Inertia;
use Inertia\ProvidesInertiaProperties;
use Inertia\RenderContext;
use Throwable;

final readonly class MapSkyhooksFeature implements ProvidesInertiaProperties
{
    /**
     * @param  string[]  $hiddenCards
     */
    public function __construct(
        private array $hiddenCards = [],
    ) {}

    public function toInertiaProperties(RenderContext $context): array
    {
        if (in_array(RemovableCard::Skyhooks->value, $this->hiddenCards)) {
            return [];
        }

        return [
            'map_skyhooks' => Inertia::defer($this->getSkyhooks(...)),
        ];
    }

    /**
     * @throws Throwable
     */
    private function getSkyhooks(): ResourceCollection
    {
        return RaidableSkyhook::query()
            ->with('planet:id,name,type_id')
            ->orderBy('theft_vulnerability_start')
            ->get()
            ->toResourceCollection(RaidableSkyhookResource::class);
    }
}
