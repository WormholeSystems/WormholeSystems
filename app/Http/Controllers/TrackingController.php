<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\Tracking\StoreTrackingAction;
use App\Data\TrackingData;
use Illuminate\Http\RedirectResponse;
use Throwable;

final class TrackingController extends Controller
{
    /**
     * @throws Throwable
     */
    public function store(TrackingData $data, StoreTrackingAction $action): RedirectResponse
    {
        $action->handle($data);

        return back();
    }
}
