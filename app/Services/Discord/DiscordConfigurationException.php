<?php

declare(strict_types=1);

namespace App\Services\Discord;

use RuntimeException;

/**
 * A server-side Discord configuration problem (e.g. missing bot token). Unlike other
 * delivery RuntimeExceptions this is never an alert's fault, so delivery jobs must
 * retry instead of disabling the alert.
 */
final class DiscordConfigurationException extends RuntimeException {}
