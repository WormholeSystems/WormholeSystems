<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;

class ServeOnlyLocalhostMiddleware
{
    /**
     * Handle an incoming request.
     *
     * @param  Closure(Request): (Response)  $next
     */
    public function handle(Request $request, Closure $next): Response
    {
        abort_unless(in_array($request->ip(), ['127.0.0.1', '::1'], true), Response::HTTP_NOT_FOUND);

        return $next($request);
    }
}
