<?php

declare(strict_types=1);

use App\Models\User;
use Illuminate\Support\Facades\Auth;

it('exposes a null email without tripping strict missing-attribute protection', function () {
    $user = User::factory()->create();

    // Retrieved (not recently created) so strict mode would throw on a truly
    // missing attribute - this is the state tooling reads the user in.
    $retrieved = User::query()->findOrFail($user->id);

    expect($retrieved->email)->toBeNull();
});

it('lets authenticated-user tooling read email during a request', function () {
    $user = User::factory()->create();

    Auth::login(User::query()->findOrFail($user->id));

    expect(Auth::user())->toBeInstanceOf(User::class)
        ->and(Auth::user()->email)->toBeNull();
});
