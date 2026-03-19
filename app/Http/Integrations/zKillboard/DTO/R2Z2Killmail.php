<?php

declare(strict_types=1);

namespace App\Http\Integrations\zKillboard\DTO;

final class R2Z2Killmail
{
    public function __construct(
        public int $killmail_id,
        public string $hash,
        public array $esi,
        public zkb $zkb,
        public int $sequence_id,
    ) {}

    public static function fromArray(array $data): self
    {
        return new self(
            killmail_id: (int) ($data['killmail_id'] ?? 0),
            hash: $data['hash'] ?? '',
            esi: $data['esi'] ?? [],
            zkb: zkb::fromArray($data['zkb'] ?? []),
            sequence_id: (int) ($data['sequence_id'] ?? 0),
        );
    }

    public function getSolarSystemId(): int
    {
        return (int) ($this->esi['solar_system_id'] ?? 0);
    }

    public function getKillmailTime(): ?string
    {
        return $this->esi['killmail_time'] ?? null;
    }

    public function getVictimShipTypeId(): int
    {
        return (int) ($this->esi['victim']['ship_type_id'] ?? 0);
    }

    public function getVictimCorporationId(): ?int
    {
        $id = $this->esi['victim']['corporation_id'] ?? null;

        return $id ? (int) $id : null;
    }

    public function getVictimAllianceId(): ?int
    {
        $id = $this->esi['victim']['alliance_id'] ?? null;

        return $id ? (int) $id : null;
    }
}
