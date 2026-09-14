<?php

declare(strict_types=1);

namespace App\Http\Controllers\Concerns;

use App\Services\Statistics\MaintainerPeriod;
use Illuminate\Http\Request;
use InvalidArgumentException;

/**
 * Shared by the stats API and the settings page controller so both read the same
 * `?period=` value the same way. Unparseable input is a 422; anything outside
 * MaintainerPeriod::selectable() -- older than the 12 previous months, or in the
 * future -- is a 404.
 */
trait HasMaintainerPeriodResolution
{
    private function resolveMaintainerPeriod(Request $request): MaintainerPeriod
    {
        $value = $request->query('period');

        if ($value !== null && ! is_string($value)) {
            abort(422, 'Invalid period.');
        }

        try {
            $period = is_string($value) && $value !== '' ? MaintainerPeriod::fromString($value) : MaintainerPeriod::current();
        } catch (InvalidArgumentException) {
            abort(422, 'Invalid period.');
        }

        $isSelectable = collect(MaintainerPeriod::selectable())
            ->contains(fn (MaintainerPeriod $selectable): bool => $selectable->toString() === $period->toString());

        if (! $isSelectable) {
            abort(404);
        }

        return $period;
    }
}
