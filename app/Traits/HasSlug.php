<?php

namespace App\Traits;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Str;

/**
 * @mixin Model
 *
 * @property-read string $slug
 */
trait HasSlug
{
    public function getSlugAttribute(): string
    {
        return Str::slug(implode('-', $this->getSlugParts()), '-');
    }

    public function getSlugParts(): array
    {
        return [
            $this->name,
            $this->id,
        ];
    }

    public function resolveRouteBinding($value, $field = null): ?Model
    {
        $id = last(explode('-', (string) $value));

        return $this->findOrFail($id);
    }

    public function getRouteKeyName(): string
    {
        return 'slug';
    }
}
