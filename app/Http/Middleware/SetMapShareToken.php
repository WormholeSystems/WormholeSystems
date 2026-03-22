<?php

declare(strict_types=1);

namespace App\Http\Middleware;

use App\Models\Map;
use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;

final class SetMapShareToken
{
    /**
     * Store the share token in the session when present in the query string.
     *
     * @param  Closure(Request): (Response)  $next
     */
    public function handle(Request $request, Closure $next): Response
    {
        $shareToken = $request->query('share_token');

        if (is_string($shareToken) && $shareToken !== '') {
            $map = $request->route('map');

            if ($map instanceof Map) {
                $request->session()->put("map_share_token_{$map->id}", $shareToken);
            }
        }

        return $next($request);
    }
}
