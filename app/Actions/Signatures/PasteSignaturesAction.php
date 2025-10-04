<?php

declare(strict_types=1);

namespace App\Actions\Signatures;

use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use App\Models\Signature;
use App\Models\SignatureCategory;
use App\Models\SignatureType;
use Illuminate\Support\Facades\DB;
use Throwable;

final readonly class PasteSignaturesAction
{
    public function __construct(
        public StoreSignatureAction $storeSignatureAction,
        public UpdateSignatureAction $updateSignatureAction,
    ) {}

    /**
     * Execute the action.
     *
     * @throws Throwable
     */
    public function handle(array $data): void
    {
        DB::transaction(function () use ($data): void {
            $map_solarsystem = MapSolarsystem::query()->findOrFail($data['map_solarsystem_id']);
            $signatures = collect($data['signatures']);
            $existing_signatures = $map_solarsystem->signatures;

            $new_signatures = $signatures->filter(fn (array $signature): bool => $existing_signatures->firstWhere('signature_id', $signature['signature_id']) === null);
            $updated_signatures = $signatures->filter(fn (array $signature): bool => $existing_signatures->firstWhere('signature_id', $signature['signature_id']) !== null);

            $new_signatures->each(fn (array $signature): Signature => $this->storeSignatureAction->handle([
                ...$signature,
                'map_solarsystem_id' => $map_solarsystem->id,
            ]));

            $updated_signatures->each(function (array $signature) use ($existing_signatures): void {
                $existing_signature = $existing_signatures->firstWhere('signature_id', $signature['signature_id']);

                $new_category = $this->getNewCategory($signature, $existing_signature);
                $new_type = $this->getNewType($signature, $existing_signature);
                $new_map_connection = $this->getNewMapConnection($new_category, $existing_signature);

                $existing_signature->update([
                    'category' => $new_category,
                    'type' => $new_type,
                    'map_connection_id' => $new_map_connection?->id,
                    'wormhole_id' => $new_type->wormhole?->id,
                ]);
            });
        });
    }

    private function getNewCategory(array $signature, Signature $existing_signature): ?SignatureCategory
    {
        if ($category = $signature['category'] ?? null) {
            return SignatureCategory::query()->firstWhere('name', $category);
        }

        return $existing_signature->signatureCategory;
    }

    private function getNewType(array $signature, Signature $existing_signature): ?SignatureType
    {
        if ($type = $signature['type'] ?? null) {
            return SignatureType::query()->firstWhere('name', $type);
        }

        return $existing_signature->signatureType;
    }

    private function getNewMapConnection(?SignatureCategory $category, Signature $existing_signature): ?MapConnection
    {
        if ($category?->name === \App\Enums\SignatureCategory::Wormhole->value) {
            return $existing_signature->mapConnection;
        }

        return null;
    }
}
