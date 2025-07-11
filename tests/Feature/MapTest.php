<?php

use App\Models\Type;

test('1', function () {

    $response = $this->get('/');

    $response->assertStatus(200);

    $this->assertTrue((bool) Type::query()->first());
});

test('2', function () {

    $response = $this->get('/');

    $response->assertStatus(200);

    $this->assertTrue((bool) Type::query()->first());
});
