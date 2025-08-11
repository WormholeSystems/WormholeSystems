<?php

declare(strict_types=1);

namespace App\Http\Requests\Signatures;

use App\Enums\SignatureCategory;
use App\Models\MapConnection;
use App\Models\Signature;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Container\Attributes\RouteParameter;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;
use Illuminate\Validation\Rule;

final class UpdateSignatureRequest extends FormRequest
{
    public ?MapConnection $mapConnection {
        get {
            if (! $this->has('map_connection_id')) {
                return null;
            }

            return MapConnection::query()->find($this->integer('map_connection_id'));
        }
    }

    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(#[CurrentUser] User $user, #[RouteParameter('signature')] Signature $signature): bool
    {
        if (! $user->can('update', $signature)) {
            return false;
        }

        if (! $this->mapConnection instanceof MapConnection) {
            return true;
        }
        if ($this->mapConnection->fromMapSolarsystem()->is($signature->mapSolarsystem)) {
            return true;
        }

        return $this->mapConnection->toMapSolarsystem()->is($signature->mapSolarsystem);
    }

    /**
     * Get the validation rules that apply to the request.
     *
     * @return array<string, ValidationRule|array|string>
     */
    public function rules(): array
    {
        return [
            'signature_id' => ['nullable', 'sometimes', 'string', 'max:7', 'min:7'],
            'category' => ['nullable', 'sometimes', Rule::enum(SignatureCategory::class)],
            'type' => ['nullable', 'sometimes', 'string', 'max:255'],
            'map_connection_id' => ['nullable', 'sometimes', 'integer', 'exists:map_connections,id'],
        ];
    }
}
