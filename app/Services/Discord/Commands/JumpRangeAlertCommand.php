<?php

declare(strict_types=1);

namespace App\Services\Discord\Commands;

use App\Enums\JumpShipType;

final readonly class JumpRangeAlertCommand implements AlertVariantDefinition
{
    use BuildsCommandOptions;

    public function definition(bool $withMentions): SubCommand
    {
        $ship = Option::string('ship')->description('Ship class')->required();
        foreach (JumpShipType::cases() as $type) {
            $ship->choice($type->label(), $type->value);
        }

        $subCommand = SubCommand::make('jump-range', 'A k-space exit lands within capital jump range of the target')->options(
            $this->mapOption(),
            $this->systemOption(),
            $ship,
            Option::integer('jdc')->description('Jump Drive Calibration level')->required()->between(1, 5),
        );

        if ($withMentions) {
            $subCommand->options(...$this->mentionOptions());
        }

        return $subCommand->options(Option::boolean('highsec')->description('Include high-sec exits'));
    }
}
