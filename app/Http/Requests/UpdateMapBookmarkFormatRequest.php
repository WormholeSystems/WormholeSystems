<?php

declare(strict_types=1);

namespace App\Http\Requests;

use App\Enums\AliasScheme;
use App\Enums\BookmarkToken;
use App\Models\Map;
use App\Models\User;
use Closure;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Container\Attributes\RouteParameter;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;
use Illuminate\Validation\Rule;

final class UpdateMapBookmarkFormatRequest extends FormRequest
{
    /**
     * The bookmark format is shared by every viewer, so only managers may change it.
     */
    public function authorize(#[RouteParameter('map')] Map $map, #[CurrentUser] User $user): bool
    {
        return $user->can('updateSettings', $map);
    }

    /**
     * @return array<string, ValidationRule|array|string>
     */
    public function rules(): array
    {
        return [
            'bookmark_format_wormhole' => ['sometimes', 'string', 'max:255', $this->onlyKnownTokens()],
            'bookmark_format_kspace' => ['sometimes', 'string', 'max:255', $this->onlyKnownTokens()],
            'bookmark_format_return' => ['sometimes', 'string', 'max:255', $this->onlyKnownTokens()],
            'bookmark_alias_scheme' => ['sometimes', Rule::enum(AliasScheme::class)],
            'bookmark_ignored_alias' => ['sometimes', 'string', 'max:255'],
        ];
    }

    /**
     * The global `ConvertEmptyStringsToNull` middleware turns a submitted empty string into
     * `null` before validation runs, but an empty ignored alias is a valid, meaningful value
     * (it disables the feature) rather than an absent one. Coerce only `null` back to an empty
     * string so `string` still validates and the column (not nullable) never receives `null`;
     * any other non-string input is left for the `string` rule to reject.
     */
    protected function prepareForValidation(): void
    {
        if ($this->has('bookmark_ignored_alias')) {
            $this->merge(['bookmark_ignored_alias' => $this->input('bookmark_ignored_alias') ?? '']);
        }
    }

    /**
     * Reject any {placeholder} that is not one of the supported bookmark tokens.
     */
    private function onlyKnownTokens(): Closure
    {
        return function (string $attribute, mixed $value, Closure $fail): void {
            if (! is_string($value)) {
                return;
            }

            preg_match_all('/\{(\w+)\}/', $value, $matches);

            $unknown = array_diff($matches[1], BookmarkToken::names());

            if ($unknown !== []) {
                $fail(sprintf('The %s contains unknown tokens: %s.', $attribute, implode(', ', array_map(static fn (string $token): string => '{'.$token.'}', $unknown))));
            }
        };
    }
}
