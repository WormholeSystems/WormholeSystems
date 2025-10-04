<?php

declare(strict_types=1);

namespace App\Actions\Signatures;

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

                $signature_category_id = $signature['signature_category_id'] ?? $existing_signature->signature_category_id;
                $signature_type_id = $signature['signature_type_id'] ?? $existing_signature->signature_type_id;

                // Clear map_connection_id if category is not Wormhole
                $map_connection_id = null;
                if ($signature_category_id) {
                    $category = SignatureCategory::query()->find($signature_category_id);
                    if ($category?->name === 'Wormhole') {
                        $map_connection_id = $existing_signature->map_connection_id;
                    }
                }

                // Get wormhole_id from signature type
                $wormhole_id = null;
                if ($signature_type_id) {
                    $signatureType = SignatureType::query()->find($signature_type_id);
                    $wormhole_id = $signatureType?->wormhole?->id;
                }

                $existing_signature->update([
                    'signature_category_id' => $signature_category_id,
                    'signature_type_id' => $signature_type_id,
                    'map_connection_id' => $map_connection_id,
                    'wormhole_id' => $wormhole_id,
                ]);
            });
        });
    }
}
