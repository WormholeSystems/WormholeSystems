<?php

declare(strict_types=1);

use App\Http\Controllers\Api\MapController;
use App\Http\Controllers\Api\MapSolarsystemController;
use Illuminate\Support\Facades\Route;

Route::middleware('auth:sanctum')->name('api.')->group(function () {
    Route::resource('maps', MapController::class)->only(['update', 'show', 'index']);
    Route::resource('map-solarsystems', MapSolarsystemController::class)->only(['show', 'update', 'store', 'destroy']);
});
