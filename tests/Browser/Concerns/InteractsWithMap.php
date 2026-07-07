<?php

declare(strict_types=1);

namespace Tests\Browser\Concerns;

use App\Models\Map;
use App\Models\MapSolarsystem;
use App\Models\MapUserSetting;
use App\Models\User;

use function Pest\Laravel\actingAs;

/**
 * Helpers for driving the map UI in browser tests, keeping the tests themselves declarative.
 */
trait InteractsWithMap
{
    /**
     * Create a map owner with a resolved active character, then act as them. The onboarding
     * wizard is pre-dismissed by default (pass false to leave it shown, e.g. to test it).
     */
    protected function actAsMapOwner(Map $map, bool $dismissOnboarding = true): User
    {
        $user = User::factory()->ownsMap($map)->create();
        $user->update(['preferred_character_id' => $user->characters->first()->id]);

        if ($dismissOnboarding) {
            MapUserSetting::query()->updateOrCreate(
                ['user_id' => $user->id, 'map_id' => $map->id],
                ['introduction_confirmed_at' => now()],
            );
        }

        actingAs($user);

        return $user;
    }

    /**
     * Place a system on the map (seeding the underlying solarsystem so the FK and the
     * client-side static lookup both resolve).
     */
    protected function placeSystem(Map $map, int $solarsystemId, int $x, int $y): MapSolarsystem
    {
        makeSolarsystem($solarsystemId);

        return MapSolarsystem::factory()->for($map)->create([
            'solarsystem_id' => $solarsystemId,
            'position_x' => $x,
            'position_y' => $y,
            'pinned' => false,
        ]);
    }

    /**
     * Perform a custom pointer drag from $source to $target.
     *
     * The map uses custom pointer events (not native HTML drag-and-drop). The browser driver's
     * drag performs the real press + move (rendering any live preview) but swallows the release
     * the interaction listens for, so the matching pointerup over the target is delivered to
     * finalise it. Pass $reveal to hover a parent first when the source only appears on hover.
     *
     * @param  string|null  $reveal  CSS selector to hover before dragging (e.g. to reveal a hover-only handle)
     */
    protected function pointerDrag(mixed $page, string $source, string $target, ?string $reveal = null): mixed
    {
        if ($reveal !== null) {
            $page->hover($reveal);
        }

        return $page
            ->drag($source, $target)
            ->script(sprintf(
                <<<'JS'
                const target = document.querySelector(%s);
                if (target) {
                    const rect = target.getBoundingClientRect();
                    target.dispatchEvent(new PointerEvent('pointerup', {
                        bubbles: true,
                        clientX: rect.left + rect.width / 2,
                        clientY: rect.top + rect.height / 2,
                    }));
                }
                JS,
                json_encode($target),
            ));
    }

    /**
     * Draw a connection from one system to another by dragging from the first system's
     * connection handle onto the second.
     */
    protected function connectSystems(mixed $page, int $fromSolarsystemId, int $toSolarsystemId): mixed
    {
        return $this->pointerDrag(
            $page,
            source: "[data-connection-source=\"{$fromSolarsystemId}\"]",
            target: "[data-solarsystem-id=\"{$toSolarsystemId}\"]",
            reveal: "[data-solarsystem-id=\"{$fromSolarsystemId}\"]",
        );
    }
}
