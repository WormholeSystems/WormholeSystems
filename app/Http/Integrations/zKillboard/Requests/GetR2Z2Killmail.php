<?php

declare(strict_types=1);

namespace App\Http\Integrations\zKillboard\Requests;

use App\Enums\RequestMethod;
use App\Http\Integrations\zKillboard\DTO\R2Z2Killmail;
use App\Http\Integrations\zKillboard\zKillboardRequest;

final class GetR2Z2Killmail extends zKillboardRequest
{
    public RequestMethod $method = RequestMethod::GET;

    public ?string $base_url = 'https://r2z2.zkillboard.com';

    public function __construct(
        public int $sequence_id,
    ) {}

    public function getEndpoint(): string
    {
        return sprintf('/ephemeral/%d.json', $this->sequence_id);
    }

    public function toDto(?array $data): ?R2Z2Killmail
    {
        if ($data === null || $data === [] || ! isset($data['killmail_id'])) {
            return null;
        }

        return R2Z2Killmail::fromArray($data);
    }
}
