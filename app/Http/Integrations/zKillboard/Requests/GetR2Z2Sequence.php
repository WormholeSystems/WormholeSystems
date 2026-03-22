<?php

declare(strict_types=1);

namespace App\Http\Integrations\zKillboard\Requests;

use App\Enums\RequestMethod;
use App\Http\Integrations\zKillboard\zKillboardRequest;

final class GetR2Z2Sequence extends zKillboardRequest
{
    public RequestMethod $method = RequestMethod::GET;

    public ?string $base_url;

    public function __construct()
    {
        $this->base_url = config()->string('services.zkillboard.r2z2_base_url');
    }

    public function getEndpoint(): string
    {
        return '/ephemeral/sequence.json';
    }

    public function toDto(?array $data): int
    {
        return (int) ($data['sequence'] ?? 0);
    }
}
