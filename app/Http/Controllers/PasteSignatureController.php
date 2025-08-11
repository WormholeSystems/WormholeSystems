<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\Signatures\PasteSignaturesAction;
use App\Http\Requests\PasteSignaturesRequest;
use Illuminate\Http\RedirectResponse;
use Throwable;

final class PasteSignatureController extends Controller
{
    /**
     * @throws Throwable
     */
    public function store(PasteSignaturesRequest $request, PasteSignaturesAction $action): RedirectResponse
    {
        $action->handle($request->validated());

        return back()->notify('Signature pasted successfully!', message: 'You successfully pasted a signature from clipboard.');
    }
}
