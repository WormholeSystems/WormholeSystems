<?php

declare(strict_types=1);

namespace App\Data;

use App\Enums\MassStatus;
use App\Enums\ShipSize;
use App\Enums\SignatureCategory;
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
        public string|Optional|null $type,
        public SignatureCategory|Optional|null $category,
        public int|Optional|null $map_connection_id,
        public MassStatus|Optional|null $mass_status,
        public ShipSize|Optional|null $ship_size,
        #[WithCast(DateTimeInterfaceCast::class)]
        public DateTimeImmutable|Optional|null $marked_as_eol_at,
    ) {}

    public static function rules(): array
    {
        return [
            'signature_id' => ['nullable', 'sometimes', 'string', 'max:7', 'min:7'],
            'category' => ['nullable', 'sometimes', Rule::enum(SignatureCategory::class)],
            'type' => ['nullable', 'sometimes', 'string', 'max:255'],
            'map_connection_id' => ['nullable', 'sometimes', 'integer', 'exists:map_connections,id'],
            'marked_as_eol_at' => ['nullable', 'sometimes', "date_format:Y-m-d\TH:i:sP"],
            'mass_status' => ['nullable', 'sometimes', Rule::enum(MassStatus::class)],
            'ship_size' => ['nullable', 'sometimes', Rule::enum(ShipSize::class)],
        ];
    }

    public static function authorize(#[CurrentUser] User $user, #[RouteParameter('signature')] Signature $signature): bool
    {
        if (! $user->can('update', $signature)) {
            return false;
        }

        $map_connection = MapConnection::find($signature->map_connection_id);

        if (! $map_connection instanceof MapConnection) {
            return true;
        }
        if ($map_connection->fromMapSolarsystem()->is($signature->mapSolarsystem)) {
            return true;
        }

        return $map_connection->toMapSolarsystem()->is($signature->mapSolarsystem);
    }
}
