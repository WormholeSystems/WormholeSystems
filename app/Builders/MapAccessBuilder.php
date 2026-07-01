<?php

declare(strict_types=1);

namespace App\Builders;

use App\Models\MapAccess;
use Illuminate\Database\Eloquent\Builder;

/**
 * @mixin MapAccess
 *
 * @template T of MapAccess
 *
 * @extends Builder<T>
 */
final class MapAccessBuilder extends Builder
{
    /**
     * Access entries that are still valid: no expiry, or an expiry still in the future.
     */
    public function notExpired(): self
    {
        return $this->where(fn (Builder $query) => $query
            ->whereNull('expires_at')
            ->orWhere('expires_at', '>', now()));
    }
}
