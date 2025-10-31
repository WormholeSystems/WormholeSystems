<?php

declare(strict_types=1);

namespace App\Features;

use App\Http\Resources\SolarsystemResource;
use App\Models\Solarsystem;
use Illuminate\Http\Resources\Json\ResourceCollection;
use Illuminate\Support\Stringable;
use Inertia\ProvidesInertiaProperties;
use Inertia\RenderContext;
use Throwable;

final readonly class MapSearchFeature implements ProvidesInertiaProperties
{
    public function __construct(
        private Stringable $search,
    ) {}

    public function toInertiaProperties(RenderContext $context): array
    {
        return [
            'search' => $this->search,
            'solarsystems' => $this->getSolarsystemsMatchingSearch(...),
        ];
    }

    /**
     * @throws Throwable
     */
    private function getSolarsystemsMatchingSearch(): ResourceCollection
    {
        return Solarsystem::query()
            ->search($this->search)
            ->limit(10)
            ->get()
            ->toResourceCollection(SolarsystemResource::class);
    }
}
