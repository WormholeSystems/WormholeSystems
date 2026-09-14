<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\Signatures\PasteSignaturesAction;
use App\Data\SignaturesData;
use App\Models\User;
use App\Support\ActingCharacter;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Http\RedirectResponse;
use Illuminate\Support\Facades\Gate;
use Throwable;

final class PasteSignatureController extends Controller
{
    public function __construct(
        #[CurrentUser] private readonly User $user
    ) {}

    /**
     * @throws Throwable
     */
    public function store(SignaturesData $data, PasteSignaturesAction $action): RedirectResponse
    {
        Gate::authorize('update', $data->mapSolarsystem);

        $action->handle($data, actor: ActingCharacter::resolve($this->user));

        return back()->notify('Signature pasted successfully!', message: 'You successfully pasted a signature from clipboard.');
    }
}
