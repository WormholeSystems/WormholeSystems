<?php

declare(strict_types=1);

namespace App\Actions\Signatures;

use App\Actions\MapConnections\SyncConnectionShipSizeAction;
use App\Data\NewSignatureData;
use App\Data\RawSignatureData;
use App\Data\SignaturesData;
use App\Enums\SignatureActivityAction;
use App\Models\Character;
use App\Models\MapSolarsystem;
use App\Models\Signature;
use App\Models\SignatureCategory;
use App\Models\SignatureType;
use App\Support\Broadcasting\MapBroadcaster;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Support\Arr;
use Illuminate\Support\Facades\DB;
use Spatie\LaravelData\Optional;
use Throwable;

final readonly class PasteSignaturesAction
{
    private SignatureCategory $wormholeCategory;

    public function __construct(
        public StoreSignatureAction $storeSignatureAction,
        public UpdateSignatureAction $updateSignatureAction,
        private MapBroadcaster $mapBroadcaster,
        private SyncConnectionShipSizeAction $syncConnectionShipSizeAction,
        private RecordSignatureActivityAction $recordSignatureActivityAction,
    ) {
        $this->wormholeCategory = SignatureCategory::query()->firstWhere('code', \App\Enums\SignatureCategory::Wormhole);
    }

    /**
     * Execute the action.
     *
     * @throws Throwable
     */
    public function handle(SignaturesData $data, ?Character $actor = null): void
    {
        DB::transaction(function () use ($data, $actor): void {
            $map_solarsystem = MapSolarsystem::query()->findOrFail($data->map_solarsystem_id);
            $signatures = collect($data->signatures);
            $existing_signatures = $map_solarsystem->signatures;

            $new_signatures = $signatures->filter(fn (RawSignatureData $signature): bool => $existing_signatures->firstWhere('signature_id', $signature->signature_id) === null);
            $updated_signatures = $signatures->filter(fn (RawSignatureData $signature): bool => $existing_signatures->firstWhere('signature_id', $signature->signature_id) !== null);

            $new_signatures->each(function (RawSignatureData $signature) use ($map_solarsystem, $actor): Signature {
                $data = [
                    'signature_id' => $signature->signature_id,
                    'signature_category_id' => $signature->signature_category_id,
                    'signature_type_id' => $signature->signature_type_id,
                ];

                if (! ($signature->raw_type_name instanceof Optional)) {
                    $data['raw_type_name'] = $signature->raw_type_name;
                }

                // Omitted -> let NewSignatureData apply its own default (false): there is no
                // prior row whose flag could be clobbered.
                if (! ($signature->is_anomaly instanceof Optional)) {
                    $data['is_anomaly'] = $signature->is_anomaly;
                }

                return $this->storeSignatureAction->handle(
                    $map_solarsystem,
                    NewSignatureData::from($data),
                    without_signatures_changed_event: true,
                    actor: $actor,
                );
            });

            $updated_signatures->each(function (RawSignatureData $signature) use ($existing_signatures, $actor): void {
                $existing_signature = $this->getExistingSignature($existing_signatures, $signature->signature_id);

                $signature_category_id = $signature->signature_category_id ?? $existing_signature->signature_category_id;

                $signature_type_id = $this->resolveSignatureTypeId($signature, $existing_signature);
                $raw_type_name = $this->resolveRawTypeName($signature, $existing_signature);

                $map_connection_id = $this->getNewMapConnectionId($signature_category_id, $existing_signature->map_connection_id);

                $wormhole_id = $this->getNewWormholeId($signature_type_id);

                // Omitted -> keep the existing flag, same as every other field above: a paste
                // that doesn't say "is_anomaly" must not silently clear it.
                $is_anomaly = $signature->is_anomaly instanceof Optional ? $existing_signature->is_anomaly : $signature->is_anomaly;

                $existing_signature->update([
                    'signature_category_id' => $signature_category_id,
                    'signature_type_id' => $signature_type_id,
                    'map_connection_id' => $map_connection_id,
                    'wormhole_id' => $wormhole_id,
                    'raw_type_name' => $raw_type_name,
                    'is_anomaly' => $is_anomaly,
                ]);

                // Snapshot before syncConnectionShipSizeAction(), which writes to the signature
                // itself when it is a typed, connected signature -- capturing after it would
                // score a point for a re-paste that changed nothing the user typed.
                $changed = Arr::except($existing_signature->getChanges(), ['updated_at']);

                $this->syncConnectionShipSizeAction->handle($existing_signature);

                if ($changed !== []) {
                    $this->recordSignatureActivityAction->handle($existing_signature, $actor, SignatureActivityAction::Updated);
                }
            });

            // A paste of N signatures emits a single counts event for the system.
            if ($signatures->isNotEmpty()) {
                $this->mapBroadcaster->signaturesChanged($map_solarsystem);
            }
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
