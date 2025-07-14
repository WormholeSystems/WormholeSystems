<?php

use App\Http\Controllers\Api\MapController;
use App\Http\Controllers\Api\MapSolarsystemController;
use App\Http\Middleware\ServeOnlyLocalhostMiddleware;
use Illuminate\Support\Facades\Route;

Route::resource('maps', MapController::class)->middleware(ServeOnlyLocalhostMiddleware::class)->only(['update', 'show', 'index']);
Route::resource('map-solarsystems', MapSolarsystemController::class)->middleware(ServeOnlyLocalhostMiddleware::class)->only(['update', 'store', 'destroy']);
