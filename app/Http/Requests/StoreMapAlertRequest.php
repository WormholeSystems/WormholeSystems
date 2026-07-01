<?php

declare(strict_types=1);

namespace App\Http\Requests;

use App\Http\Requests\Concerns\HasMapAlertRules;
use App\Models\Map;
use App\Models\MapAlert;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

final class StoreMapAlertRequest extends FormRequest
{
    use HasMapAlertRules;

    public Map $map {
        get => Map::query()->findOrFail(
            $this->integer('map_id'),
        );
    }

    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(#[CurrentUser] User $user): bool
    {
        return $user->can('create', [MapAlert::class, $this->map]);
    }

    /**
     * Get the validation rules that apply to the request.
     *
     * @return array<string, ValidationRule|array|string>
     */
    public function rules(): array
    {
        return [
            'map_id' => ['required', 'integer', 'exists:maps,id'],
            ...$this->alertRules(),
        ];
    }

    protected function alertMap(): Map
    {
        return $this->map;
    }
}
