<?php

declare(strict_types=1);

namespace App\Scopes;

use App\Enums\MassStatus;
use Illuminate\Database\Eloquent\Builder;

final class ConnectionSatisfiesMass
{
    public function __construct(
        private MassStatus $massStatus,
    ) {}

    public function __invoke(Builder $query)
    {
        return $query
            ->when($this->massStatus === MassStatus::Fresh, fn (Builder $query) => $query->where('mass_status', MassStatus::Fresh))
            ->when($this->massStatus === MassStatus::Reduced, fn (Builder $query) => $query->whereIn('mass_status', [MassStatus::Fresh, MassStatus::Reduced]));
    }
}
