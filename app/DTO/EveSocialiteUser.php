<?php

declare(strict_types=1);

namespace App\DTO;

use SocialiteProviders\Manager\OAuth2\User;

class EveSocialiteUser
{
    public function __construct(
        public int $character_id,
        public string $character_name,
        public string $character_owner_hash,
        public string $token,
        public string $refresh_token,
        public string $token_type,
        public int $expires_in,
        public array $scopes,
    ) {}

    public static function fromSocialiteUser(\Laravel\Socialite\Contracts\User $user): self
    {
        assert($user instanceof User);

        $scopes = $user->user['scp'] ?? [];
        $scopes = is_array($scopes) ? $scopes : [$scopes];

        return new self(
            character_id: (int) $user->attributes['character_id'],
            character_name: $user->attributes['character_name'],
            character_owner_hash: $user->attributes['character_owner_hash'],
            token: $user->token,
            refresh_token: $user->refreshToken,
            token_type: $user->accessTokenResponseBody['token_type'],
            expires_in: $user->accessTokenResponseBody['expires_in'],
            scopes: $scopes,
        );
    }
}
