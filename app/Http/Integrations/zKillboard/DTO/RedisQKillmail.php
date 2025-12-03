<?php

declare(strict_types=1);

namespace App\Http\Integrations\zKillboard\DTO;

final class RedisQKillmail
{
    public function __construct(
        public int $killID,
        public zkb $zkb,
    ) {}

    public static function fromArray(array $data): self
    {
        $killID = $data['killID'] ?? 0;
        $zkb = zkb::fromArray($data['zkb'] ?? []);

        return new self(
            killID: (int) $killID,
            zkb: $zkb,
        );
    }
}
