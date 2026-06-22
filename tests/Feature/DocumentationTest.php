<?php

declare(strict_types=1);

it('renders the bookmarking cheat sheet page', function () {
    $response = $this->get(route('documentation', 'bookmarking/cheat-sheet'));

    $response->assertSuccessful();
    $response->assertInertia(fn ($page) => $page
        ->component('Documentation')
        ->where('page.slug', 'bookmarking/cheat-sheet')
        ->where('page.title', 'Cheat sheet')
        ->where('page.category', 'Bookmarking')
    );
});

it('lists the cheat sheet as the first page under bookmarking', function () {
    $response = $this->get(route('documentation', 'bookmarking/cheat-sheet'));

    $response->assertSuccessful();
    $response->assertInertia(function ($page) {
        $navigation = collect($page->toArray()['props']['navigation']);

        $bookmarking = $navigation->firstWhere('title', 'Bookmarking');

        expect($bookmarking)->not->toBeNull();
        expect($bookmarking['pages'][0]['slug'])->toBe('bookmarking/cheat-sheet');
    });
});
