<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Http\Requests\EveSearchRequest;
use App\Models\Group;
use App\Models\Type;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Http\JsonResponse;

/**
 * Read-only typeahead over the static EVE reference data, used by the killmail webhook
 * filter editor to look up types and groups by name (or resolve ids to names when
 * editing). Unfiltered by category or published flag — structures, deployables, fighters
 * and unpublished NPC/officer entities (e.g. "Asteroid Serpentis Officer") all turn up on
 * killmails, so all of them must be searchable.
 */
final class EveSearchController extends Controller
{
    private const int LIMIT = 25;

    public function index(EveSearchRequest $request): JsonResponse
    {
        $ids = array_map(intval(...), $request->input('ids', []) ?? []);
        $query = mb_trim((string) $request->string('q'));

        if ($ids === [] && $query === '') {
            return response()->json(['data' => []]);
        }

        $results = $request->string('kind')->value() === 'group'
            ? $this->groups($ids, $query)
            : $this->types($ids, $query);

        return response()->json(['data' => $results]);
    }

    /**
     * @param  int[]  $ids
     * @return array<int, array{id: int, name: string, group_name: string|null}>
     */
    private function types(array $ids, string $query): array
    {
        return $this->matching(Type::query()->with('group:id,name'), $ids, $query)
            ->get(['id', 'name', 'group_id'])
            ->map(fn (Type $type): array => [
                'id' => $type->id,
                'name' => $type->name,
                'group_name' => $type->group?->name,
            ])
            ->all();
    }

    /**
     * @param  int[]  $ids
     * @return array<int, array{id: int, name: string, category_name: string|null}>
     */
    private function groups(array $ids, string $query): array
    {
        return $this->matching(Group::query()->with('category:id,name'), $ids, $query)
            ->get(['id', 'name', 'category_id'])
            ->map(fn (Group $group): array => [
                'id' => $group->id,
                'name' => $group->name,
                'category_name' => $group->category?->name,
            ])
            ->all();
    }

    /**
     * Resolve the given ids, or substring-search by name when no ids are given. Search
     * results are ranked by relevance (exact match, then prefix, then substring) so the
     * closest match isn't pushed past the limit by alphabetically-earlier partial hits.
     *
     * @param  int[]  $ids
     */
    private function matching(Builder $builder, array $ids, string $query): Builder
    {
        // Resolving explicit ids (labelling saved chips when editing) must return every
        // one of them; the limit only guards the open-ended typeahead search below.
        if ($ids !== []) {
            return $builder->whereIn('id', $ids)->orderBy('name');
        }

        return $builder
            ->whereLike('name', sprintf('%%%s%%', $query))
            ->orderByRaw(
                'CASE WHEN name = ? THEN 0 WHEN name LIKE ? THEN 1 ELSE 2 END, CHAR_LENGTH(name), name',
                [$query, sprintf('%s%%', $query)],
            )
            ->limit(self::LIMIT);
    }
}
