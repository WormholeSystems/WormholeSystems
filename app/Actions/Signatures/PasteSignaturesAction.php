<?php

declare(strict_types=1);

namespace App\Actions\Signatures;

use App\Models\MapSolarsystem;
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

            $new_signatures = $signatures->filter(fn (array $signature) => $existing_signatures->firstWhere('signature_id', $signature['signature_id']) === null);
            $updated_signatures = $signatures->filter(fn (array $signature) => $existing_signatures->firstWhere('signature_id', $signature['signature_id']) !== null);

            $new_signatures->each(fn (array $signature) => $this->storeSignatureAction->handle([
                ...$signature,
                'map_solarsystem_id' => $map_solarsystem->id,
            ]));

            $updated_signatures->each(function (array $signature) use ($existing_signatures): void {
                $existing_signature = $existing_signatures->firstWhere('signature_id', $signature['signature_id']);

                $new_category = empty($signature['category']) ? $existing_signature->category : $signature['category'];
                $new_type = empty($signature['type']) ? $existing_signature->type : $signature['type'];
                $new_map_connection_id = $new_category !== 'Wormhole' ? null : $existing_signature->map_connection_id;

                $existing_signature->update([
                    'category' => $new_category,
                    'type' => $new_type,
                    'map_connection_id' => $new_map_connection_id,
                ]);
            });
        });
    }
}
