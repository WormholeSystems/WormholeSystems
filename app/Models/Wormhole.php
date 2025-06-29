<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;

class Wormhole extends Model
{
    protected $fillable = [
        'name',
        'total_mass',
        'maximum_jump_mass',
        'ship_size',
        'maximum_lifetime',
        'leads_to',
    ];
}
