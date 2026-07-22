<?php

declare(strict_types=1);

namespace App\Services\Discord\Commands;

use Discord\Parts\Interactions\ApplicationCommand;
use Discord\Parts\Interactions\Request\InteractionData;
use Discord\Parts\Interactions\Request\Option;

trait ReadsInteractionOptions
{
    private function subcommand(ApplicationCommand $interaction): Option
    {
        return $interaction->data->options->first();
    }

    private function option(Option|InteractionData $source, string $name): mixed
    {
        return $source->options->get('name', $name)?->value;
    }
}
