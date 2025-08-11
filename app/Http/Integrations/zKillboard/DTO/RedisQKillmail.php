<?php

declare(strict_types=1);

namespace App\Http\Integrations\zKillboard\DTO;

use NicolasKion\Esi\DTO\Killmail;

final class RedisQKillmail
{
    public function __construct(
        public int $killID,
        public Killmail $killmail,
        public zkb $zkb,
    ) {}

    public static function fromArray(array $data): self
    {
        $killID = $data['killID'] ?? 0;
        $killmail = Killmail::fromArray($data['killmail'] ?? []);
        $zkb = zkb::fromArray($data['zkb'] ?? []);

        return new self(
            killID: $killID,
            killmail: $killmail,
            zkb: $zkb,
        );
    }
}
