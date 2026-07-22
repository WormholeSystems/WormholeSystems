<?php

declare(strict_types=1);

namespace App\Http\Requests;

use App\Enums\MapAlertDeliveryType;
use App\Http\Requests\Concerns\HasMapAlertRules;
use App\Models\Map;
use App\Models\MapAlert;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

final class UpdateMapAlertRequest extends FormRequest
{
    use HasMapAlertRules;

    public MapAlert $mapAlert {
        get => $this->route('map_alert');
    }

    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(#[CurrentUser] User $user): bool
    {
        return $this->mapAlert->delivery_type === MapAlertDeliveryType::Webhook
            && $user->can('update', $this->mapAlert);
    }

    /**
     * Get the validation rules that apply to the request.
     *
     * @return array<string, ValidationRule|array|string>
     */
    public function rules(): array
    {
        return $this->alertRules();
    }

    protected function alertMap(): Map
    {
        return $this->mapAlert->map;
    }
}
