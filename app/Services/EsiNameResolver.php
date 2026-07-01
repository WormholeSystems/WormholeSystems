<?php

declare(strict_types=1);

namespace App\Services;

use Illuminate\Support\Facades\Cache;
use NicolasKion\Esi\DTO\Name;
use NicolasKion\Esi\Esi;
use Throwable;

/**
 * Resolves EVE entity ids (characters, corporations, alliances, …) to their names via
 * ESI's bulk names endpoint, caching each name forever since they effectively never
 * change. Resolution is best-effort: ids that can't be resolved are simply omitted.
 */
final class EsiNameResolver implements NameResolver
{
    public function __construct(
        private readonly Esi $esi,
    ) {}

    /**
     * @param  int[]  $ids
     * @return array<int, string>
     */
    public function resolve(array $ids): array
    {
        $ids = array_values(array_unique(array_filter($ids, fn (int $id): bool => $id > 0)));

        if ($ids === []) {
            return [];
        }

        $names = [];
        $missing = [];

        foreach ($ids as $id) {
            $cached = Cache::get('esi-name:'.$id);

            if (is_string($cached)) {
                $names[$id] = $cached;
            } else {
                $missing[] = $id;
            }
        }

        if ($missing !== []) {
            $this->fetch($missing, $names);
        }

        return $names;
    }

    /**
     * @param  int[]  $ids
     * @param  array<int, string>  $names
     */
    private function fetch(array $ids, array &$names): void
    {
        try {
            $result = $this->esi->getNames($ids);

            if ($result->failed()) {
                return;
            }

            foreach ($result->data as $name) {
                $names[$name->id] = $name->name;
                Cache::forever('esi-name:'.$name->id, $name->name);
            }
        } catch (Throwable) {
            // Best-effort: leave unresolved ids out of the result.
        }
    }
}
