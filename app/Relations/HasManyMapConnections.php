<?php

declare(strict_types=1);

namespace App\Relations;

use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\Relation;

/**
 * Custom relation that combines both connectionsFrom and connectionsTo
 * Returns all MapConnections where the parent MapSolarsystem is either
 * the from_map_solarsystem_id or to_map_solarsystem_id
 *
 * @extends Relation<MapConnection, MapSolarsystem, Collection<int, MapConnection>>
 *
 * @method Builder<MapConnection> getQuery()
 * @method Collection<int, MapConnection> get(array|string $columns = ['*'])
 * @method MapConnection|null first(array|string $columns = ['*'])
 * @method bool exists()
 * @method int count(string $columns = '*')
 * @method mixed sum(string $column)
 * @method mixed avg(string $column)
 * @method mixed min(string $column)
 * @method mixed max(string $column)
 */
final class HasManyMapConnections extends Relation
{
    /**
     * Set the base constraints on the relation query.
     */
    public function addConstraints(): void
    {
        if (self::$constraints) {
            $this->query->where(function ($query): void {
                $query->where('from_map_solarsystem_id', '=', $this->parent->getKey())
                    ->orWhere('to_map_solarsystem_id', '=', $this->parent->getKey());
            });
        }
    }

    /**
     * Set the constraints for an eager load of the relation.
     *
     * @param  array<int, MapSolarsystem>  $models
     */
    public function addEagerConstraints(array $models): void
    {
        $keys = $this->getKeys($models, $this->parent->getKeyName());

        $this->query->where(function ($query) use ($keys): void {
            $query->whereIn('from_map_solarsystem_id', $keys)
                ->orWhereIn('to_map_solarsystem_id', $keys);
        });
    }

    /**
     * Initialize the relation on a set of models.
     *
     * @param  array<int, Model>  $models
     * @param  string  $relation
     * @return array<int, Model>
     */
    public function initRelation(array $models, $relation): array
    {
        foreach ($models as $model) {
            $model->setRelation($relation, $this->related->newCollection());
        }

        return $models;
    }

    /**
     * Match the eagerly loaded results to their parents.
     *
     * @param  array<int, MapSolarsystem>  $models
     * @param  Collection<int, MapConnection>  $results
     * @param  string  $relation
     * @return array<int, Model>
     */
    public function match(array $models, Collection $results, $relation): array
    {
        $dictionary = $this->buildDictionary($results);

        // Once we have the dictionary we can simply spin through the parent models to
        // link them up with their children using the keyed dictionary to make the
        // matching very convenient and easy work. Then we'll just return them.
        foreach ($models as $model) {
            $key = $model->getKey();

            if (isset($dictionary[$key])) {
                $model->setRelation($relation, $this->related->newCollection($dictionary[$key]));
            }
        }

        return $models;
    }

    /**
     * Get the results of the relationship.
     *
     * @return Collection<int, MapConnection>
     */
    public function getResults(): Collection
    {
        return $this->query->get();
    }

    /**
     * Add the constraints for a relationship count / exists query.
     *
     * @param  Builder<MapConnection>  $query
     * @param  Builder<MapSolarsystem>  $parentQuery
     * @param  array|mixed  $columns
     * @return Builder<MapConnection>
     */
    public function getRelationExistenceQuery(Builder $query, Builder $parentQuery, $columns = ['*'])
    {
        if ($query->getQuery()->from === $parentQuery->getQuery()->from) {
            return $this->getRelationExistenceQueryForSelfRelation($query, $parentQuery, $columns);
        }

        $parentTable = $parentQuery->getModel()->getTable();
        $parentKey = $parentQuery->getModel()->getKeyName();
        $qualifiedParentKey = $parentTable.'.'.$parentKey;

        $connectionTable = $query->getModel()->getTable();

        return $query->select($columns)->where(function ($q) use ($qualifiedParentKey, $connectionTable): void {
            $q->whereColumn($connectionTable.'.from_map_solarsystem_id', '=', $qualifiedParentKey)
                ->orWhereColumn($connectionTable.'.to_map_solarsystem_id', '=', $qualifiedParentKey);
        });
    }

    /**
     * Add the constraints for a relationship query on the same table.
     *
     * @param  Builder<MapConnection>  $query
     * @param  Builder<MapSolarsystem>  $parentQuery
     * @param  array|mixed  $columns
     * @return Builder<MapConnection>
     */
    public function getRelationExistenceQueryForSelfRelation(Builder $query, Builder $parentQuery, $columns = ['*'])
    {
        $query->from($query->getModel()->getTable().' as '.$hash = $this->getRelationCountHash());

        $query->getModel()->setTable($hash);

        $parentKey = $parentQuery->getModel()->getQualifiedKeyName();

        return $query->select($columns)->where(function ($query) use ($hash, $parentKey): void {
            $query->whereColumn($hash.'.from_map_solarsystem_id', '=', $parentKey)
                ->orWhereColumn($hash.'.to_map_solarsystem_id', '=', $parentKey);
        });
    }

    /**
     * Build model dictionary keyed by the relation's foreign key.
     *
     * @param  Collection<int, MapConnection>  $results
     * @return array<int, array<int, MapConnection>>
     */
    private function buildDictionary(Collection $results): array
    {
        $dictionary = [];

        foreach ($results as $result) {
            // Add to dictionary for from_map_solarsystem_id
            $fromKey = $result->getAttribute('from_map_solarsystem_id');
            if ($fromKey !== null) {
                $dictionary[$fromKey][] = $result;
            }

            // Add to dictionary for to_map_solarsystem_id
            $toKey = $result->getAttribute('to_map_solarsystem_id');
            if ($toKey !== null) {
                $dictionary[$toKey][] = $result;
            }
        }

        return $dictionary;
    }
}
