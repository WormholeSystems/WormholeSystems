<?php

declare(strict_types=1);

use App\Models\User;

it('renders the landing page for guests', function () {
    $response = $this->get(route('landing'));

    $response->assertSuccessful();
    $response->assertInertia(fn ($page) => $page->component('Landing'));
});

it('redirects authenticated users away from the landing page', function () {
    $this->actingAs(User::factory()->create());

    $response = $this->get(route('landing'));

    $response->assertRedirect();
});
