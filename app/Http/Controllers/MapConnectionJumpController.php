<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\MapConnections\CreateMapConnectionJumpAction;
use App\Actions\MapConnections\DeleteMapConnectionJumpAction;
use App\Actions\MapConnections\UpdateMapConnectionJumpAction;
use App\Http\Requests\StoreMapConnectionJumpRequest;
use App\Http\Requests\UpdateMapConnectionJumpRequest;
use App\Models\MapConnectionJump;
use Illuminate\Http\RedirectResponse;
use Illuminate\Support\Facades\Gate;

final class MapConnectionJumpController extends Controller
{
    public function store(StoreMapConnectionJumpRequest $request, CreateMapConnectionJumpAction $action): RedirectResponse
    {
        $action->handle($request->validated());

        return back()->notify(
            'Jump logged!',
            'The jump has been added to the connection\'s mass estimate.'
        );
    }

    public function update(UpdateMapConnectionJumpRequest $request, MapConnectionJump $mapConnectionJump, UpdateMapConnectionJumpAction $action): RedirectResponse
    {
        $action->handle($mapConnectionJump, $request->validated());

        return back()->notify(
            'Jump updated!',
            'The jump entry has been updated.'
        );
    }

    public function destroy(MapConnectionJump $mapConnectionJump, DeleteMapConnectionJumpAction $action): RedirectResponse
    {
        Gate::authorize('delete', $mapConnectionJump);

        $action->handle($mapConnectionJump);

        return back()->notify(
            'Jump removed!',
            'The jump entry has been removed from the connection.'
        );
    }
}
