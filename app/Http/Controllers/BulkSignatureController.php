<?php

namespace App\Http\Controllers;

use App\Actions\DeleteSignaturesAction;
use App\Models\MapSolarsystem;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Gate;
use Throwable;

class BulkSignatureController extends Controller
{
    /**
     * @throws Throwable
     */
    public function destroy(Request $request, MapSolarsystem $mapSolarsystem, DeleteSignaturesAction $deleteSignaturesAction)
    {
        Gate::authorize('update', $mapSolarsystem);

        $deleteSignaturesAction->handle(
            mapSolarsystem: $mapSolarsystem,
            signature_ids: $request->input('signature_ids', []),
        );

        return back()->notify(
            'Signatures deleted successfully!',
            message: 'You successfully deleted the selected signatures.',
        );
    }
}
