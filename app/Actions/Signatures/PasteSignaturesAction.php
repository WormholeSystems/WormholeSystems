<?php

declare(strict_types=1);

namespace App\Actions\Signatures;

use App\Data\NewSignatureData;
use App\Data\RawSignatureData;
use App\Data\SignaturesData;
use App\Models\MapSolarsystem;
use App\Models\Signature;
use App\Models\SignatureCategory;
use App\Models\SignatureType;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Support\Facades\DB;
use Spatie\LaravelData\Optional;
use Throwable;

final readonly class PasteSignaturesAction
{
    private SignatureCategory $wormholeCategory;

    public function __construct(
        public StoreSignatureAction $storeSignatureAction,
        public UpdateSignatureAction $updateSignatureAction,
    ) {
        $this->wormholeCategory = SignatureCategory::query()->firstWhere('code', \App\Enums\SignatureCategory::Wormhole);
    }

    /**
     * Execute the action.
     *
     * @throws Throwable
     */
    public function handle(SignaturesData $data): void
    {
        DB::transaction(function () use ($data): void {
            $map_solarsystem = MapSolarsystem::query()->findOrFail($data->map_solarsystem_id);
            $signatures = collect($data->signatures);
            $existing_signatures = $map_solarsystem->signatures;

            $new_signatures = $signatures->filter(fn (RawSignatureData $signature): bool => $existing_signatures->firstWhere('signature_id', $signature->signature_id) === null);
            $updated_signatures = $signatures->filter(fn (RawSignatureData $signature): bool => $existing_signatures->firstWhere('signature_id', $signature->signature_id) !== null);

            $new_signatures->each(function (RawSignatureData $signature) use ($map_solarsystem): Signature {
                $data = [
                    'signature_id' => $signature->signature_id,
                    'signature_category_id' => $signature->signature_category_id,
                    'signature_type_id' => $signature->signature_type_id,
                ];

                if (! ($signature->raw_type_name instanceof Optional)) {
                    $data['raw_type_name'] = $signature->raw_type_name;
                }

                return $this->storeSignatureAction->handle(
                    $map_solarsystem,
                    NewSignatureData::from($data)
                );
            });

            $updated_signatures->each(function (RawSignatureData $signature) use ($existing_signatures): void {
                $existing_signature = $this->getExistingSignature($existing_signatures, $signature->signature_id);

                $signature_category_id = $signature->signature_category_id ?? $existing_signature->signature_category_id;

                $signature_type_id = $this->resolveSignatureTypeId($signature, $existing_signature);
                $raw_type_name = $this->resolveRawTypeName($signature, $existing_signature);

                $map_connection_id = $this->getNewMapConnectionId($signature_category_id, $existing_signature->map_connection_id);

                $wormhole_id = $this->getNewWormholeId($signature_type_id);

                $existing_signature->update([
                    'signature_category_id' => $signature_category_id,
                    'signature_type_id' => $signature_type_id,
                    'map_connection_id' => $map_connection_id,
                    'wormhole_id' => $wormhole_id,
                    'raw_type_name' => $raw_type_name,
                ]);
            });
        });
    }

    /**
     * @param  Collection<int, Signature>  $signatures
     */
    private function getExistingSignature(Collection $signatures, string $id): ?Signature
    {
        return $signatures->firstWhere('signature_id', $id);
    }

    private function getNewMapConnectionId(?int $signature_category_id, ?int $existing_map_connection_id): ?int
    {
        if ($signature_category_id === null) {
            return $existing_map_connection_id;
        }

        $category = SignatureCategory::query()->find($signature_category_id);
        if ($category?->name === 'Wormhole') {
            return $existing_map_connection_id;
        }

        return null;
    }

    private function getNewWormholeId(?int $signature_type_id): ?int
    {
        if ($signature_type_id === null) {
            return null;
        }

        $type = SignatureType::query()->find($signature_type_id);

        return $type?->wormhole?->id;
    }

    /**
     * Resolve signature type ID with mutual exclusivity.
     *
     * Returns null if raw_type_name is present (temporary event site),
     * otherwise returns the new or existing type ID.
     */
    private function resolveSignatureTypeId(RawSignatureData $signature, Signature $existingSignature): ?int
    {
        $has_raw_type_name = ! ($signature->raw_type_name instanceof Optional) && $signature->raw_type_name !== null;

        if ($signature->signature_category_id !== $this->wormholeCategory->id && $has_raw_type_name) {
            return null;
        }

        return $signature->signature_type_id ?? $existingSignature->signature_type_id;
    }

    /**
     * Resolve raw type name with mutual exclusivity.
     *
     * Returns null if signature_type_id is present (known type from database),
     * otherwise returns the new or existing raw type name.
     */
    private function resolveRawTypeName(RawSignatureData $signature, Signature $existingSignature): ?string
    {
        if ($signature->signature_category_id === $this->wormholeCategory->id) {
            return null;
        }

        if ($signature->raw_type_name instanceof Optional || $signature->raw_type_name === null) {
            return $existingSignature->raw_type_name;
        }

        return $signature->raw_type_name;
    }
}
