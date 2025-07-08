<?php

namespace App\Http\Controllers;

use App\Actions\Signatures\StoreSignaturesAction;
use App\Http\Requests\Signatures\StoreSignaturesRequest;
use App\Models\MapSolarsystem;
use Illuminate\Http\RedirectResponse;

class SignatureController extends Controller
{
    public function store(StoreSignaturesRequest $request, MapSolarsystem $mapSolarsystem, StoreSignaturesAction $storeSignaturesAction): RedirectResponse
    {
        $storeSignaturesAction->handle($mapSolarsystem, $request->validated()['signatures']);

        return back();
    }
}
