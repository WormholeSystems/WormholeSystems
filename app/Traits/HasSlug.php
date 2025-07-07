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
    public string $slug {
        get => Str::slug(implode('-', $this->getSlugParts()));
    }

    public function getSlugParts(): array
    {
        return [
            $this->getSlugName(),
            $this->getSlugKey(),
        ];
    }

    public function getSlugKey(): string
    {
        return $this->getKeyName();
    }

    public function getSlugName(): string
    {
        return $this->name;
    }
}
