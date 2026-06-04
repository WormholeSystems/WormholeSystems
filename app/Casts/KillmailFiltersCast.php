<?php

declare(strict_types=1);

namespace App\Casts;

use App\Services\Killmails\KillmailFilterRule;
use Illuminate\Contracts\Database\Eloquent\CastsAttributes;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Collection;

/**
 * Casts the `filters` json column to a collection of {@see KillmailFilterRule} DTOs
 * and back. A null column or empty list both yield an empty collection.
 *
 * @implements CastsAttributes<Collection<int, KillmailFilterRule>, iterable<int, KillmailFilterRule|array>>
 */
final class KillmailFiltersCast implements CastsAttributes
{
    /**
     * @param  array<array-key, mixed>  $attributes
     * @return Collection<int, KillmailFilterRule>
     */
    public function get(Model $model, string $key, mixed $value, array $attributes): Collection
    {
        if ($value === null) {
            return new Collection;
        }

        $decoded = json_decode((string) $value, true);

        if (! is_array($decoded)) {
            return new Collection;
        }

        return new Collection($decoded)->map(KillmailFilterRule::fromArray(...));
    }

    /**
     * @param  array<array-key, mixed>  $attributes
     * @param  iterable<int, KillmailFilterRule|array>|null  $value
     */
    public function set(Model $model, string $key, mixed $value, array $attributes): ?string
    {
        if ($value === null) {
            return null;
        }

        $rules = new Collection($value)->map(
            fn (KillmailFilterRule|array $rule): array => $rule instanceof KillmailFilterRule
                ? $rule->toArray()
                : KillmailFilterRule::fromArray($rule)->toArray(),
        );

        return json_encode($rules->values()->all());
    }
}
