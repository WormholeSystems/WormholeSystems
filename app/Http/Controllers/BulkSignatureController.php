<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\DeleteSignaturesAction;
use App\Models\MapSolarsystem;
use App\Models\User;
use App\Support\ActingCharacter;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Gate;
use Throwable;

final class BulkSignatureController extends Controller
{
    public function __construct(
        #[CurrentUser] private readonly User $user
    ) {}

    /**
     * @throws Throwable
     */
    public function destroy(Request $request, MapSolarsystem $mapSolarsystem, DeleteSignaturesAction $deleteSignaturesAction)
    {
        Gate::authorize('update', $mapSolarsystem);

        $deleteSignaturesAction->handle(
            mapSolarsystem: $mapSolarsystem,
            signature_ids: $request->input('signature_ids', []),
            remove_map_solarsystems: $request->boolean('remove_map_solarsystems'),
            actor: ActingCharacter::resolve($this->user),
        );

        return back()->notify(
            'Signatures deleted successfully!',
            message: 'You successfully deleted the selected signatures.',
        );
    }
}
