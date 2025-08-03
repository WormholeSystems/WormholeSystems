<?php

namespace App\Scopes;

use App\Enums\MassStatus;
use Illuminate\Database\Eloquent\Builder;

class ConnectionSatisfiesMass
{
    public function __construct(
        protected MassStatus $massStatus,
    ) {}

    public function __invoke(Builder $query)
    {
        return $query
            ->when($this->massStatus === MassStatus::Fresh, fn (Builder $query) => $query->where('mass_status', MassStatus::Fresh))
            ->when($this->massStatus === MassStatus::Reduced, fn (Builder $query) => $query->whereIn('mass_status', [MassStatus::Fresh, MassStatus::Reduced]));
    }
}
