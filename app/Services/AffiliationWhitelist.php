<?php

declare(strict_types=1);

namespace App\Services;

use Illuminate\Container\Attributes\Config;

final readonly class AffiliationWhitelist
{
    /**
     * @param  list<int>  $allowedIds
     */
    public function __construct(
        #[Config('access.allowed_affiliation_ids')]
        private array $allowedIds = [],
    ) {}

    /**
     * Whether the whitelist is configured and should be enforced.
     */
    public function isEnforced(): bool
    {
        return $this->allowedIds !== [];
    }

    /**
     * Determine whether any of the given affiliation IDs is allowed to log in.
     *
     * When the whitelist is empty, every affiliation is allowed.
     *
     * @param  array<int|null>  $affiliationIds
     */
    public function allows(array $affiliationIds): bool
    {
        if (! $this->isEnforced()) {
            return true;
        }

        return array_intersect($this->allowedIds, array_filter($affiliationIds)) !== [];
    }
}
