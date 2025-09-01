<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\Signatures\DeleteSignatureAction;
use App\Actions\Signatures\StoreSignatureAction;
use App\Actions\Signatures\UpdateSignatureAction;
use App\Data\SignatureData;
use App\Http\Requests\Signatures\StoreSignatureRequest;
use App\Models\Signature;
use Illuminate\Http\RedirectResponse;
use Illuminate\Support\Facades\Gate;
use Throwable;

final class SignatureController extends Controller
{
    /**
     * @throws Throwable
     */
    public function store(StoreSignatureRequest $request, StoreSignatureAction $storeSignatureAction): RedirectResponse
    {
        $storeSignatureAction->handle($request->validated());

        return back()->notify('Signature created successfully.', message: 'You have successfully created a new signature.');
    }

    /**
     * @throws Throwable
     */
    public function update(Signature $signature, SignatureData $signatureData, UpdateSignatureAction $updateSignatureAction): RedirectResponse
    {
        $updateSignatureAction->handle($signature, $signatureData);

        return back()->notify('Signature updated successfully.', message: 'You have successfully updated the signature.');
    }

    /**
     * @throws Throwable
     */
    public function destroy(Signature $signature, DeleteSignatureAction $deleteSignatureAction): RedirectResponse
    {
        Gate::authorize('delete', $signature);

        $deleteSignatureAction->handle($signature);

        return back()->notify('Signature deleted successfully.', message: 'You have successfully deleted the signature.');
    }
}
