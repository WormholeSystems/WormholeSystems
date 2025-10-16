<?php

declare(strict_types=1);

namespace App\Data;

use App\Enums\LifetimeStatus;
use App\Enums\MassStatus;
use App\Enums\ShipSize;
use App\Models\MapConnection;
use App\Models\Signature;
use App\Models\User;
use DateTimeImmutable;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Container\Attributes\RouteParameter;
use Illuminate\Validation\Rule;
use Spatie\LaravelData\Attributes\WithCast;
use Spatie\LaravelData\Casts\DateTimeInterfaceCast;
use Spatie\LaravelData\Data;
use Spatie\LaravelData\Optional;

final class SignatureData extends Data
{
    public function __construct(
        public string|Optional|null $signature_id,
        public int|Optional|null $signature_type_id,
        public int|Optional|null $signature_category_id,
        public int|Optional|null $map_connection_id,
        public MassStatus|Optional|null $mass_status,
        public ShipSize|Optional|null $ship_size,
        public LifetimeStatus|Optional $lifetime,
        #[WithCast(DateTimeInterfaceCast::class)]
        public DateTimeImmutable|Optional|null $lifetime_updated_at,
        public string|Optional|null $raw_type_name,
    ) {}

    public static function rules(): array
    {
        return [
            'signature_id' => ['nullable', 'sometimes', 'string', 'max:7', 'min:7'],
            'signature_category_id' => ['nullable', 'sometimes', 'integer', 'exists:signature_categories,id'],
            'signature_type_id' => ['nullable', 'sometimes', 'integer', 'exists:signature_types,id'],
            'map_connection_id' => ['nullable', 'sometimes', 'integer', 'exists:map_connections,id'],
            'lifetime' => ['nullable', 'sometimes', Rule::enum(LifetimeStatus::class)],
            'lifetime_updated_at' => ['nullable', 'sometimes', "date_format:Y-m-d\TH:i:sP"],
            'mass_status' => ['nullable', 'sometimes', Rule::enum(MassStatus::class)],
            'ship_size' => ['nullable', 'sometimes', Rule::enum(ShipSize::class)],
            'raw_type_name' => ['nullable', 'sometimes', 'string', 'max:255'],
        ];
    }

    public static function authorize(#[CurrentUser] User $user, #[RouteParameter('signature')] Signature $signature): bool
    {
        if (! $user->can('update', $signature)) {
            return false;
        }

        $map_connection = MapConnection::query()->find($signature->map_connection_id);

        if (! $map_connection instanceof MapConnection) {
            return true;
        }
        if ($map_connection->fromMapSolarsystem()->is($signature->mapSolarsystem)) {
            return true;
        }

        return $map_connection->toMapSolarsystem()->is($signature->mapSolarsystem);
    }
}
