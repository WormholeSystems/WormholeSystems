<?php

namespace App\Actions\Map;

use App\Models\Map;

class CreateMapAction
{
    /**
     * Create a new class instance.
     */
    public function __construct()
    {
        //
    }

    public function handle(array $data): Map
    {
        return Map::create($data);
    }
}
