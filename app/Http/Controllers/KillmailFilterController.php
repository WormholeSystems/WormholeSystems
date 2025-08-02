<?php

namespace App\Http\Controllers;

use App\Enums\KillmailFilter;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Session;

class KillmailFilterController extends Controller
{
    public function update(Request $request): RedirectResponse
    {
        $filter = $request->enum('killmail_filter', KillmailFilter::class);

        if ($filter === null) {
            return back()->notify('Invalid filter', message: 'The selected filter is not valid.', type: 'error');
        }

        Session::put('killmail_filter', $filter->value);

        return back()->notify('Filter updated', message: 'The killmail filter has been updated successfully.');
    }
}
