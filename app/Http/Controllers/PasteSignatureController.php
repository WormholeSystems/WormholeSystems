<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\Signatures\PasteSignaturesAction;
use App\Data\SignaturesData;
use Illuminate\Http\RedirectResponse;
use Illuminate\Support\Facades\Gate;
use Throwable;

final class PasteSignatureController extends Controller
{
    /**
     * @throws Throwable
     */
    public function store(SignaturesData $data, PasteSignaturesAction $action): RedirectResponse
    {
        Gate::authorize('update', $data->mapSolarsystem);

        $action->handle($data);

        return back()->notify('Signature pasted successfully!', message: 'You successfully pasted a signature from clipboard.');
    }
}
