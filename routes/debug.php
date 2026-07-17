<?php

declare(strict_types=1);

use Illuminate\Support\Facades\Route;
use Inertia\Inertia;
use Inertia\Response;

/*
 * Debug-only routes. Registered exclusively in local and testing environments;
 * see the routing configuration in bootstrap/app.php.
 */

Route::get('debug/tracking-signature-dialog', fn (): Response => Inertia::render('debug/TrackingSignatureDialogPreview', [
    'map' => ['slug' => 'debug-preview'],
]))->name('debug.tracking-signature-dialog');
