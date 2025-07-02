<?php

use App\Http\Controllers\MapConnectionController;
use App\Http\Controllers\MapController;
use App\Http\Controllers\MapSolarsystemController;
use Illuminate\Support\Facades\Route;
use Inertia\Inertia;

Route::get('/', function () {
    return Inertia::render('Welcome');
})->name('home');

Route::get('dashboard', function () {
    return Inertia::render('Dashboard');
})->middleware(['auth', 'verified'])->name('dashboard');

require __DIR__.'/settings.php';
require __DIR__.'/auth.php';

Route::resource('maps', MapController::class);

Route::resource('map-solarsystems', MapSolarsystemController::class)->only(['store', 'update', 'destroy']);
Route::resource('map-connections', MapConnectionController::class)->only(['store', 'update', 'destroy']);
