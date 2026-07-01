<?php

declare(strict_types=1);

namespace App\Services\Killmails;

use App\Enums\KillmailFilterMode;
use App\Enums\KillmailFilterSide;
use App\Enums\KillmailFilterSubject;

/**
 * A single criterion a killmail webhook evaluates. Multiple ids within one rule are
 * OR'd together; rules are combined by the matcher (every include rule must match,
 * no exclude rule may match).
 */
final readonly class KillmailFilterRule
{
    /**
     * @param  int[]  $ids
     */
    public function __construct(
        public KillmailFilterSubject $subject,
        public KillmailFilterSide $side,
        public KillmailFilterMode $mode,
        public array $ids,
    ) {}

    /**
     * @param  array{subject: string, side: string, mode: string, ids: array<int|string>}  $data
     */
    public static function fromArray(array $data): self
    {
        return new self(
            subject: KillmailFilterSubject::from($data['subject']),
            side: KillmailFilterSide::from($data['side']),
            mode: KillmailFilterMode::from($data['mode']),
            ids: array_values(array_map(intval(...), $data['ids'])),
        );
    }

    /**
     * @return array{subject: string, side: string, mode: string, ids: int[]}
     */
    public function toArray(): array
    {
        return [
            'subject' => $this->subject->value,
            'side' => $this->side->value,
            'mode' => $this->mode->value,
            'ids' => $this->ids,
        ];
    }
}
