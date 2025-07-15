<?php

namespace App\Http\Controllers;

use App\Actions\CreateStatisticsAction;
use App\Http\Requests\CreateStatisticsRequest;
use Illuminate\Http\RedirectResponse;

class StatisticsController extends Controller
{
    public function store(CreateStatisticsRequest $request, CreateStatisticsAction $action): RedirectResponse
    {
        $action->handle($request->validated());

        return back()->notify('Analysis started!', 'Your statistics analysis has been queued and will be processed shortly.');
    }
}
