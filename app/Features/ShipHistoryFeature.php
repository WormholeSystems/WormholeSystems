<?php

declare(strict_types=1);

namespace App\Features;

use App\Http\Resources\ShipHistoryResource;
use App\Models\CharacterStatus;
use App\Models\ShipHistory;
use App\Models\User;
use Illuminate\Http\Resources\Json\ResourceCollection;
use Inertia\ProvidesInertiaProperties;
use Inertia\RenderContext;
use Throwable;

final readonly class ShipHistoryFeature implements ProvidesInertiaProperties
{
    public function __construct(
        private User $user,
        private bool $canViewCharacters,
    ) {}

    public function toInertiaProperties(RenderContext $context): array
    {
        return [
            'ship_history' => fn (): ?ResourceCollection => $this->canViewCharacters ? $this->getShipHistory() : null,
        ];
    }

    /**
     * @throws Throwable
     */
    private function getShipHistory(): ResourceCollection
    {
        return ShipHistory::query()
            ->where('ship_id', '=', CharacterStatus::query()
                ->where('character_id', '=', $this->user->active_character->id)
                ->select('ship_item_id'))
            ->latest()
            ->limit(10)
            ->get()
            ->toResourceCollection(ShipHistoryResource::class);
    }
}
