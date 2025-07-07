<?php

namespace App\Http\Integrations\zKillboard;

use App\Enums\RequestMethod;
use Illuminate\Support\Facades\Http;

class zKillboardConnector
{
    public function handle(zKillboardRequest $zKillboardRequest)
    {
        $method = $zKillboardRequest->method;
        $endpoint = $zKillboardRequest->getEndpoint();
        $base_url = $zKillboardRequest->base_url ?? 'https://zkillboard.com/api';

        $request = Http::baseUrl($base_url)
            ->retry(5, 1000)
            ->withHeaders([
                'Accept' => 'application/json',
                'User-Agent' => config('esi.user_agent'),
            ]);

        $result = match ($method) {
            RequestMethod::GET => $request->get($endpoint, $zKillboardRequest->getQuery()),
            default => throw new \InvalidArgumentException('Unsupported request method: '.$method->value),
        };

        if ($result->failed()) {
            throw new \Exception('Request failed with status code: '.$result->status());
        }
        $data = $result->json();

        return $zKillboardRequest->toDto($data);
    }
}
