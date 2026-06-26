<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\MapConnections\CleanStaleMapConnectionsAction;
use App\Models\Map;
use Illuminate\Http\RedirectResponse;
use Illuminate\Support\Facades\Gate;
use Throwable;

final class BulkMapConnectionController extends Controller
{
    /**
     * @throws Throwable
     */
    public function destroy(Map $map, CleanStaleMapConnectionsAction $action): RedirectResponse
    {
        Gate::authorize('update', $map);

        $count = $action->handle($map);

        return back()->notify(
            'Map cleaned!',
            sprintf('Removed %d stale connection%s.', $count, $count === 1 ? '' : 's'),
        );
    }
}
