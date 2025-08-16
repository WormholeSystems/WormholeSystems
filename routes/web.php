
<?php

use App\Http\Controllers\AuthController;
use App\Http\Controllers\BulkSignatureController;
use App\Http\Controllers\EveController;
use App\Http\Controllers\IgnoreListController;
use App\Http\Controllers\LandingController;
use App\Http\Controllers\LoginController;
use App\Http\Controllers\MapAccessController;
use App\Http\Controllers\MapConnectionController;
use App\Http\Controllers\MapController;
use App\Http\Controllers\MapPreferencesController;
use App\Http\Controllers\MapRouteSolarsystemController;
use App\Http\Controllers\MapRoutingSettingsController;
use App\Http\Controllers\MapSelectionController;
use App\Http\Controllers\MapSettingsController;
use App\Http\Controllers\MapSolarsystemController;
use App\Http\Controllers\MapUserSettingController;
use App\Http\Controllers\PasteSignatureController;
use App\Http\Controllers\PingController;
use App\Http\Controllers\SignatureController;
use App\Http\Controllers\StatisticsController;
use App\Http\Controllers\TokenManagementController;
use App\Http\Controllers\TrackingController;
use App\Http\Controllers\UserCharacterController;
use App\Http\Controllers\WaypointController;
use Illuminate\Support\Facades\Route;

Route::get('/', [LandingController::class, 'index'])->name('landing')->middleware('guest');
Route::get('login', [LoginController::class, 'show'])->name('login');
Route::get('auth', [AuthController::class, 'show'])->name('auth');
Route::get('eve', [EveController::class, 'show'])->name('eve.show');
Route::get('eve/callback', [EveController::class, 'store'])->name('eve.store');

Route::middleware('auth')->group(function () {

    Route::get('maps/{map}/ping', [PingController::class, 'show'])->name('maps.ping');

    Route::resource('maps', MapController::class)->names([
        'index' => 'home',
        'show' => 'maps.show',
        'create' => 'maps.create',
        'store' => 'maps.store',
        'edit' => 'maps.edit',
        'update' => 'maps.update',
        'destroy' => 'maps.destroy',
    ]);

    Route::prefix('maps/{map}/settings')->name('maps.settings.')->group(function () {
        Route::get('general', [MapSettingsController::class, 'show'])->name('general.show');
        Route::get('preferences', [MapPreferencesController::class, 'show'])->name('preferences.show');

        Route::get('access', [MapAccessController::class, 'show'])->name('access.show');
        Route::post('access', [MapAccessController::class, 'store'])->name('access.store');

        Route::get('routing', [MapRoutingSettingsController::class, 'show'])->name('routing.show');
    });

    Route::delete('logout', [AuthController::class, 'destroy'])->name('logout');

    Route::resource('map-solarsystems', MapSolarsystemController::class)->only(['store', 'update', 'destroy']);
    Route::resource('map-connections', MapConnectionController::class)->only(['store', 'update', 'destroy']);
    Route::put('map-selection', [MapSelectionController::class, 'update'])->name('map-selection.update');
    Route::delete('map-selection', [MapSelectionController::class, 'destroy'])->name('map-selection.destroy');

    Route::resource('signatures', SignatureController::class)->only(['store', 'update', 'destroy']);
    Route::resource('paste-signatures', PasteSignatureController::class)->only(['store']);

    Route::put('user-characters/{character}', [UserCharacterController::class, 'update'])->name('user-characters.update');
    Route::delete('user-characters/{character}', [UserCharacterController::class, 'delete'])->name('user-characters.delete');

    Route::post('tracking', [TrackingController::class, 'store'])->name('tracking.store');

    Route::post('waypoints', [WaypointController::class, 'store'])->name('waypoints.store');

    Route::resource('map-route-solarsystems', MapRouteSolarsystemController::class)->only(['store', 'update', 'destroy']);
    Route::resource('map-user-settings', MapUserSettingController::class)->only(['update']);

    Route::post('statistics', [StatisticsController::class, 'store'])->name('statistics.store');

    Route::post('ignore-systems', [IgnoreListController::class, 'store'])->name('ignore-systems.store');
    Route::delete('ignore-system/{solarsystem_id}', [IgnoreListController::class, 'destroy'])->name('ignore-systems.destroy');
    Route::delete('ignore-systems', [IgnoreListController::class, 'destroyAll'])->name('ignore-systems.destroy-all');

    Route::delete('map-solarsystems/{mapSolarsystem}/signatures', [BulkSignatureController::class, 'destroy'])
        ->name('map-solarsystems.signatures.destroy');

    Route::resource('tokens', TokenManagementController::class)->only(['index', 'store', 'destroy']);
});
