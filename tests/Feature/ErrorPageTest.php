<?php

declare(strict_types=1);

/**
 * An unmatched route throws before the web middleware group runs, so the
 * request carries no session. The shared Inertia props must survive that.
 */
it('renders the error page for an unmatched route', function () {
    $response = $this->get('/definitely-not-a-route');

    $response->assertNotFound();
    $response->assertInertia(fn ($page) => $page
        ->component('errors/NotFound')
        ->where('status', 404));
});
