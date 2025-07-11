<?php

use App\Http\Controllers\AuthController;
use App\Http\Controllers\EveController;
use App\Http\Controllers\LandingController;
use App\Http\Controllers\LoginController;
use App\Http\Controllers\MapAccessController;
use App\Http\Controllers\MapConnectionController;
use App\Http\Controllers\MapController;
use App\Http\Controllers\MapSelectionController;
use App\Http\Controllers\MapSolarsystemController;
use App\Http\Controllers\SignatureController;
use App\Http\Controllers\TrackingController;
use App\Http\Controllers\UserCharacterController;
use Illuminate\Support\Facades\Route;

Route::get('/', [LandingController::class, 'index'])->name('landing');
Route::get('login', [LoginController::class, 'show'])->name('login');
Route::get('auth', [AuthController::class, 'show'])->name('auth');
Route::get('eve', [EveController::class, 'show'])->name('eve.show');
Route::get('eve/callback', [EveController::class, 'store'])->name('eve.store');

Route::middleware('auth')->group(function () {
    Route::resource('maps', MapController::class)->names([
        'index' => 'home',
    ]);

    Route::delete('logout', [AuthController::class, 'destroy'])->name('logout');

    Route::resource('map-solarsystems', MapSolarsystemController::class)->only(['store', 'update', 'destroy']);
    Route::resource('map-connections', MapConnectionController::class)->only(['store', 'update', 'destroy']);
    Route::put('map-selection', [MapSelectionController::class, 'update'])->name('map-selection.update');
    Route::delete('map-selection', [MapSelectionController::class, 'destroy'])->name('map-selection.destroy');

    Route::post('map-solarsystems/{mapSolarsystem}/signatures', [SignatureController::class, 'store'])->name('map-solarsystems.signatures.store');

    Route::put('user-characters/{character}', [UserCharacterController::class, 'update'])->name('user-characters.update');
    Route::delete('user-characters/{character}', [UserCharacterController::class, 'delete'])->name('user-characters.delete');

    Route::get('map-access/{map}', [MapAccessController::class, 'show'])->name('map-access.show');
    Route::post('map-access/{map}', [MapAccessController::class, 'store'])->name('map-access.store');

    Route::post('tracking', [TrackingController::class, 'store'])->name('tracking.store');
});
