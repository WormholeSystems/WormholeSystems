<?php

declare(strict_types=1);

namespace App\Http\Integrations\zKillboard\Requests;

use App\Enums\RequestMethod;
use App\Http\Integrations\zKillboard\zKillboardRequest;

final class GetSolarsystemKills extends zKillboardRequest
{
    public RequestMethod $method = RequestMethod::GET;

    public function __construct(
        public int $solarsystem_id,
    ) {}

    public function getEndpoint(): string
    {
        return sprintf('/systemID/%d/', $this->solarsystem_id);
    }

    public function getQuery(): array
    {
        return [];
    }

    public function toDto(?array $data): array
    {
        if (is_null($data)) {
            return [];
        }

        return $data;
    }
}
