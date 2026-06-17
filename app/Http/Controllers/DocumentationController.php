<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Services\DocumentationService;
use Inertia\Inertia;
use Inertia\Response;

final class DocumentationController extends Controller
{
    public function __construct(private readonly DocumentationService $documentation) {}

    public function index(?string $path = null): Response
    {
        return Inertia::render('Documentation', [
            'navigation' => $this->documentation->navigation(),
            'page' => $this->documentation->page($path),
        ]);
    }
}
