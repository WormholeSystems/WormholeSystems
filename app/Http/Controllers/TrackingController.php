<?php

namespace App\Http\Controllers;

use App\Actions\Tracking\StoreTrackingAction;
use App\Http\Requests\StoreTrackingRequest;
use Illuminate\Http\RedirectResponse;
use Throwable;

class TrackingController extends Controller
{
    /**
     * @throws Throwable
     */
    public function store(StoreTrackingRequest $request, StoreTrackingAction $action): RedirectResponse
    {
        $action->handle($request->validated());

        return back();
    }
}
