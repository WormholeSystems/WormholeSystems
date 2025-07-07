<?php

declare(strict_types=1);

use App\Models\Alliance;
use App\Models\Attribute;
use App\Models\Bloodline;
use App\Models\Category;
use App\Models\Celestial;
use App\Models\Constellation;
use App\Models\Corporation;
use App\Models\Faction;
use App\Models\Flag;
use App\Models\Graphic;
use App\Models\Group;
use App\Models\Icon;
use App\Models\MarketGroup;
use App\Models\MetaGroup;
use App\Models\Race;
use App\Models\Region;
use App\Models\Solarsystem;
use App\Models\Station;
use App\Models\Type;
use App\Models\TypeAttribute;
use App\Models\Unit;

return [
    'models' => [
        'Attribute' => Attribute::class,
        'Category' => Category::class,
        'Celestial' => Celestial::class,
        'Constellation' => Constellation::class,
        'Graphic' => Graphic::class,
        'Group' => Group::class,
        'Icon' => Icon::class,
        'MarketGroup' => MarketGroup::class,
        'MetaGroup' => MetaGroup::class,
        'Race' => Race::class,
        'Region' => Region::class,
        'Solarsystem' => Solarsystem::class,
        'Station' => Station::class,
        'Type' => Type::class,
        'TypeAttribute' => TypeAttribute::class,
        'Unit' => Unit::class,
        'Alliance' => Alliance::class,
        'Corporation' => Corporation::class,
        'Bloodline' => Bloodline::class,
        'Faction' => Faction::class,
        'Flag' => Flag::class,
    ],
];
