<?php

declare(strict_types=1);

namespace App\Data;

use App\Enums\Permission;
use App\Models\Map;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Http\Request;
use Spatie\LaravelData\Attributes\Validation\Exists;
use Spatie\LaravelData\Data;
use Spatie\LaravelData\Optional;

final class TrackingData extends Data
{
    public function __construct(
        #[Exists(table: 'map_solarsystems', column: 'id')]
        public int $from_map_solarsystem_id,
        #[Exists(table: 'solarsystems', column: 'id')]
        public int $to_solarsystem_id,
        #[Exists(table: 'signatures', column: 'id')]
        public int|null|Optional $signature_id = null,
    ) {}

    /**
     * Determine if the user is authorized to make this request.
     */
    public static function authorize(#[CurrentUser] User $user, Request $request): bool
    {
        $map = Map::query()->whereRelation('mapSolarsystems', 'id', $request->input('from_map_solarsystem_id'))->firstOrFail();

        return $map->mapAccessors()
            ->notExpired()
            ->whereIn('accessible_id', $user->getAccessibleIds())
            ->whereIn('permission', [Permission::Member, Permission::Manager])
            ->exists();
    }
}
