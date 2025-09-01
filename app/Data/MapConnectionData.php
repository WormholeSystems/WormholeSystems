<?php

declare(strict_types=1);

namespace App\Data;

use App\Enums\MassStatus;
use App\Enums\ShipSize;
use App\Models\MapConnection;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Container\Attributes\RouteParameter;
use Illuminate\Validation\Rule;
use Spatie\LaravelData\Data;
use Spatie\LaravelData\Optional;

final class MapConnectionData extends Data
{
    public function __construct(
        public int|Optional|null $wormhole_id,
        public MassStatus|Optional|null $mass_status,
        public ShipSize|Optional|null $ship_size,
        public bool|Optional|null $is_eol,
    ) {}

    public static function rules(): array
    {
        return [
            'wormhole_id' => ['nullable', 'sometimes', 'integer', 'exists:wormholes,id'],
            'mass_status' => ['nullable', 'sometimes', Rule::enum(MassStatus::class)],
            'ship_size' => ['nullable', 'sometimes', Rule::enum(ShipSize::class)],
            'is_eol' => ['nullable', 'sometimes', 'boolean'],
        ];
    }

    public static function authorize(#[CurrentUser] User $user, #[RouteParameter('map_connection')] MapConnection $mapConnection): bool
    {
        return $user->can('update', $mapConnection);
    }
}
