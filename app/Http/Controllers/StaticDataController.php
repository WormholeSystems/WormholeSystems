<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use Illuminate\Support\Facades\File;
use Symfony\Component\HttpFoundation\BinaryFileResponse;

final class StaticDataController extends Controller
{
    public function sovereignty(): BinaryFileResponse
    {
        $path = storage_path('app/static/sovereignty.json.gz');

        if (! File::exists($path)) {
            abort(404);
        }

        return response()->file($path, [
            'Cache-Control' => 'public, max-age=86400',
            'Content-Encoding' => 'gzip',
            'Content-Type' => 'application/json',
        ]);
    }
}
