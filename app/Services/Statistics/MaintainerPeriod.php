<?php

declare(strict_types=1);

namespace App\Services\Statistics;

use Carbon\CarbonImmutable;
use InvalidArgumentException;

/**
 * A UTC calendar month, e.g. "2026-06". Config('app.timezone') is UTC (§0.4), so no
 * timezone juggling is needed anywhere this is built or compared.
 */
final readonly class MaintainerPeriod
{
    /** How many periods MaintainerPeriod::selectable() returns: the current month plus the previous 12. */
    private const int SELECTABLE_COUNT = 13;

    private function __construct(
        public int $year,
        public int $month,
    ) {}

    public static function fromString(string $value): self
    {
        if (preg_match('/^(?<year>\d{4})-(?<month>\d{2})$/', $value, $matches) !== 1) {
            throw new InvalidArgumentException("Invalid maintainer period: \"{$value}\".");
        }

        $month = (int) $matches['month'];

        if ($month < 1 || $month > 12) {
            throw new InvalidArgumentException("Invalid maintainer period: \"{$value}\".");
        }

        return new self((int) $matches['year'], $month);
    }

    public static function current(): self
    {
        $now = CarbonImmutable::now('UTC');

        return new self($now->year, $now->month);
    }

    /**
     * The current month plus the previous 12, newest first -- 13 entries. This is the
     * selector's range and the API's accept-window; S4 and S6 both derive their bounds
     * from this method rather than recomputing "12 months ago".
     *
     * @return list<self>
     */
    public static function selectable(): array
    {
        $periods = [];
        $period = self::current();

        for ($i = 0; $i < self::SELECTABLE_COUNT; $i++) {
            $periods[] = $period;
            $period = $period->previous();
        }

        return $periods;
    }

    public function startsAt(): CarbonImmutable
    {
        return CarbonImmutable::create($this->year, $this->month, 1, 0, 0, 0, 'UTC');
    }

    /**
     * Exclusive end of the period -- the first instant of the following month.
     */
    public function endsAt(): CarbonImmutable
    {
        return $this->startsAt()->addMonthNoOverflow();
    }

    public function toString(): string
    {
        return sprintf('%04d-%02d', $this->year, $this->month);
    }

    public function label(): string
    {
        return $this->startsAt()->format('F Y');
    }

    public function isCurrent(): bool
    {
        return $this->toString() === self::current()->toString();
    }

    public function previous(): self
    {
        $previous = $this->startsAt()->subMonthNoOverflow();

        return new self($previous->year, $previous->month);
    }
}
