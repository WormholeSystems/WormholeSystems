<?php

declare(strict_types=1);

namespace App\Services\Discord\Commands;

final readonly class ProximityAlertCommand implements AlertVariantDefinition
{
    use BuildsCommandOptions;

    public function definition(bool $withMentions): SubCommand
    {
        $subCommand = SubCommand::make('proximity', 'A system comes within gate jumps of the target')->options(
            $this->mapOption(),
            $this->systemOption(),
            $this->jumpsOption(),
        );

        return $withMentions ? $subCommand->options(...$this->mentionOptions()) : $subCommand;
    }
}
