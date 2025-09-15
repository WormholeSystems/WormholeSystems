<?php

declare(strict_types=1);

it('shows the homepage', function () {
    visit('/')
        ->assertSee('WormholeSystems')
        ->assertSee('Sign In')
        ->assertSee('Sign in without scopes')
        ->assertSee('Join the Discord')
        ->assertNoSmoke()
        ->assertNoConsoleLogs();
});
