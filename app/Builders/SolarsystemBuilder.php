<?php

declare(strict_types=1);

namespace App\Builders;

use App\Models\Solarsystem;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Support\Stringable;

use function is_numeric;
use function sprintf;

/**
 * @mixin Solarsystem
 *
 * @template T of Solarsystem
 *
 * @extends Builder<T>
 */
final class SolarsystemBuilder extends Builder
{
    public function search(Stringable $search): self
    {
        return $this
            ->searchName($search)
            ->when(
                is_numeric($search->toString()),
                fn (self $query) => $query->orWhere(fn (self $query): SolarsystemBuilder => $query->searchJName($search))
            );
    }

    public function searchName(Stringable $search): self
    {
        return $this->whereLike('name', sprintf('%s%%', $search));
    }

    public function searchJName(Stringable $search): self
    {
        return $this->whereLike('name', sprintf('J%s%%', $search));
    }
}
