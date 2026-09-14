<?php

declare(strict_types=1);

namespace App\DTO;

use App\Models\Map;

final readonly class MaintainerSettings
{
    public function __construct(
        public int $points_created,
        public int $points_updated,
        public int $points_deleted,
        /**
         * Discord-recap-only cutoff. The leaderboard page and API list every scorer
         * regardless of this value; only PostMaintainerPodiumCommand applies it.
         */
        public int $minimum_points,
    ) {}

    public static function fromMap(Map $map): self
    {
        return new self(
            points_created: $map->maintainer_points_created,
            points_updated: $map->maintainer_points_updated,
            points_deleted: $map->maintainer_points_deleted,
            minimum_points: $map->maintainer_minimum_points,
        );
    }

    /**
     * @param  array{points_created: int, points_updated: int, points_deleted: int, minimum_points: int}  $data
     */
    public static function fromArray(array $data): self
    {
        return new self(
            points_created: $data['points_created'],
            points_updated: $data['points_updated'],
            points_deleted: $data['points_deleted'],
            minimum_points: $data['minimum_points'],
        );
    }

    /**
     * @return array{points_created: int, points_updated: int, points_deleted: int, minimum_points: int}
     */
    public function toArray(): array
    {
        return [
            'points_created' => $this->points_created,
            'points_updated' => $this->points_updated,
            'points_deleted' => $this->points_deleted,
            'minimum_points' => $this->minimum_points,
        ];
    }
}
