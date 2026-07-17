<?php

declare(strict_types=1);

it('renders the tracking signature dialog preview outside production', function () {
    $response = $this->get(route('debug.tracking-signature-dialog'));

    $response->assertSuccessful();
    $response->assertInertia(fn ($page) => $page
        ->component('debug/TrackingSignatureDialogPreview')
        ->where('map.slug', 'debug-preview'));
});
