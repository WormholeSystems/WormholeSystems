<?php

declare(strict_types=1);

namespace App\Enums;

enum BookmarkToken: string
{
    case Alias = 'alias';
    case Sig = 'sig';
    case SystemClass = 'class';
    case Name = 'name';
    case Region = 'region';
    case Occupier = 'occupier';
    case Size = 'size';
    case Wormhole = 'wh';
    case Mass = 'mass';
    case Life = 'life';

    /**
     * The default bookmark format for wormhole systems (e.g. "Home ABC C3").
     */
    public const string DEFAULT_WORMHOLE = '{alias} {sig} {class}';

    /**
     * The default bookmark format for k-space systems (e.g. "Home HS ABC Jita The Forge").
     */
    public const string DEFAULT_KSPACE = '{alias} {class} {sig} {name} {region}';

    /**
     * The default bookmark format for a return (up-chain / home) connection (e.g. "*Home ABC C3").
     * The leading "*" sorts the bookmark to the top of the in-game folder.
     */
    public const string DEFAULT_RETURN = '*{alias} {sig} {class}';

    /**
     * The default alias that is excluded as a suggestion prefix (e.g. the home system).
     */
    public const string DEFAULT_IGNORED_ALIAS = 'HOME';

    /**
     * Every placeholder name that may appear inside a bookmark format template.
     *
     * @return list<string>
     */
    public static function names(): array
    {
        return array_map(static fn (self $token): string => $token->value, self::cases());
    }
}
