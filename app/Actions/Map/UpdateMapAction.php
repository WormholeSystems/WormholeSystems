<?php

namespace App\Actions\Map;

use App\Models\Map;

class UpdateMapAction
{
    /**
     * Create a new class instance.
     */
    public function __construct()
    {
        //
    }

    public function handle(Map $map, array $data): Map
    {
        return tap($map)->update($data);
    }
}
