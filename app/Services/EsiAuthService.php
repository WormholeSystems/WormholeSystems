<?php

declare(strict_types=1);

namespace App\Services;

use App\DTO\EveSocialiteUser;
use App\Jobs\UpdateAffiliations;
use App\Models\Character;
use App\Models\Corporation;
use App\Models\EsiScope;
use App\Models\User;
use Exception;
use Illuminate\Http\Client\ConnectionException;
use Laravel\Socialite\Facades\Socialite;
use NicolasKion\Esi\DTO\CharacterAffiliation;
use NicolasKion\Esi\Esi;

final class EsiAuthService
{
    public function __construct(
        private Esi $esi
    ) {}

    /**
     * @return array{0: ?User, 1: ?Character}
     *
     * @throws ConnectionException
     */
    public function getUser(?int $add_to_user_id = null): array
    {
        $socialite_user = $this->getSocialiteUser();

        if (! $socialite_user) {
            return [
                null,
                null,
            ];
        }

        $affiliations = $this->esi->getAffiliations([$socialite_user->character_id]);

        if ($affiliations->failed() || $affiliations->data === []) {
            return [
                null,
                null,
            ];
        }

        UpdateAffiliations::dispatchSync($affiliations->data[0]);

        $character = $this->resolveCharacter($socialite_user, $affiliations->data[0]);

        $this->createEsiToken($socialite_user, $character);

        if ($add_to_user_id !== null && $add_to_user_id !== 0) {
            return [
                $this->addToAccount($character, $add_to_user_id),
                $character,
            ];
        }

        // Check if the owner hashes match

        // If they do, return the user
        if ($character->user()->exists() && $character->character_owner_hash === $socialite_user->character_owner_hash) {
            return [
                $character->user,
                $character,
            ];
        }

        return [
            $this->createNewUser($socialite_user, $character),
            $character,
        ];
    }

    public function getSocialiteUser(): EveSocialiteUser|false
    {
        try {
            $data = Socialite::driver('eveonline')->user();
        } catch (Exception) {
            return false;
        }

        return EveSocialiteUser::fromSocialiteUser($data);
    }

    public function resolveCharacter(EveSocialiteUser $socialite_user, CharacterAffiliation $affiliation): Character
    {
        $this->enureSocialsExists($affiliation);

        $character = Character::query()->updateOrCreate([
            'id' => $socialite_user->character_id,
        ], [
            'id' => $socialite_user->character_id,
            'name' => $socialite_user->character_name,
            'alliance_id' => $affiliation->alliance_id,
            'corporation_id' => $affiliation->corporation_id,
        ]);

        $character->characterStatus()->firstOrCreate();

        return $character;
    }

    public function createEsiToken(EveSocialiteUser $socialite_user, Character $character): void
    {
        $token = $character->esiTokens()->create([
            'access_token' => $socialite_user->token,
            'refresh_token' => $socialite_user->refresh_token,
            'token_type' => $socialite_user->token_type,
            'character_owner_hash' => $socialite_user->character_owner_hash,
            'expires_at' => now()->addSeconds($socialite_user->expires_in),
        ]);

        $token->esiScopes()->sync(
            EsiScope::query()->whereIn('name', $socialite_user->scopes)->pluck('id')
        );
    }

    public function addToAccount(Character $character, int $user_id): User
    {
        // Check if the user exists
        $user = User::query()->findOrFail($user_id);

        $character->user()->associate($user);
        $character->save();

        return $user;
    }

    public function createNewUser(EveSocialiteUser $socialite_user, Character $character): User
    {
        // If they don't, create a new user and link the character to it
        $user = User::query()->create([
            'name' => $socialite_user->character_name,
        ]);

        $character->user()->associate($user);
        $character->character_owner_hash = $socialite_user->character_owner_hash;
        $character->save();

        return $user;
    }

    public function enureSocialsExists(CharacterAffiliation $affiliation): void
    {
        if ($affiliation->corporation_id !== 0) {
            Corporation::query()->updateOrCreate([
                'id' => $affiliation->corporation_id,
            ]);
        }

        if ($affiliation->alliance_id !== null && $affiliation->alliance_id !== 0) {
            \App\Models\Alliance::query()->updateOrCreate([
                'id' => $affiliation->alliance_id,
            ]);
        }
    }
}
