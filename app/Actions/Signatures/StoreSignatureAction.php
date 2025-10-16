<?php

declare(strict_types=1);

namespace App\Actions\Signatures;

use App\Data\NewSignatureData;
use App\Events\Signatures\SignatureCreatedEvent;
use App\Models\MapSolarsystem;
use App\Models\Signature;
use App\Models\SignatureType;
use Illuminate\Support\Facades\DB;
use Spatie\LaravelData\Optional;
use Throwable;

final class StoreSignatureAction
{
    /**
     * @throws Throwable
     */
    public function handle(MapSolarsystem $mapSolarsystem, NewSignatureData $data): Signature
    {
        return DB::transaction(function () use ($mapSolarsystem, $data) {

            $signature_id = $data->signature_id instanceof Optional ? null : $data->signature_id;
            $signature_category_id = $data->signature_category_id instanceof Optional ? null : $data->signature_category_id;
            $signature_type_id = $data->signature_type_id instanceof Optional ? null : $data->signature_type_id;
            $map_connection_id = $data->map_connection_id instanceof Optional ? null : $data->map_connection_id;
            $raw_type_name = $data->raw_type_name instanceof Optional ? null : $data->raw_type_name;
            $wormhole_id = $this->getWormholeId($signature_type_id);

            $signature = $mapSolarsystem->signatures()->create([
                'map_connection_id' => $map_connection_id,
                'signature_id' => $signature_id,
                'signature_category_id' => $signature_category_id,
                'signature_type_id' => $signature_type_id,
                'wormhole_id' => $wormhole_id,
                'raw_type_name' => $raw_type_name,
            ]);

            broadcast(new SignatureCreatedEvent($mapSolarsystem->map_id))->toOthers();

            return $signature;
        }, attempts: 5);
    }

    private function getWormholeId(?int $signatureTypeId): ?int
    {
        if ($signatureTypeId === null) {
            return null;
        }

        $signature_type = SignatureType::query()->find($signatureTypeId);

        return $signature_type?->wormhole?->id;
    }
}
