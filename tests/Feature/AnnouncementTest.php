<?php

declare(strict_types=1);

use App\DTO\Announcement;
use App\Enums\AnnouncementLevel;

function setAnnouncementConfig(array $overrides = []): void
{
    config([
        'announcement.level' => 'info',
        'announcement.title' => null,
        'announcement.message' => null,
        'announcement.link_url' => null,
        'announcement.link_label' => null,
        'announcement.dismissible' => true,
        ...$overrides,
    ]);
}

it('shares no announcement when the message is empty', function () {
    setAnnouncementConfig();

    $this->get(route('landing'))
        ->assertSuccessful()
        ->assertInertia(fn ($page) => $page->where('announcement', null));
});

it('shares no announcement when the message is only whitespace', function () {
    setAnnouncementConfig(['announcement.message' => "  \n "]);

    expect(Announcement::fromConfig())->toBeNull();
});

it('shares the configured announcement with every page', function () {
    setAnnouncementConfig([
        'announcement.level' => 'warning',
        'announcement.title' => 'Moving to a new address',
        'announcement.message' => 'This page moves to legacy.wormhole.systems on October 1st.',
        'announcement.link_url' => 'https://legacy.wormhole.systems',
        'announcement.link_label' => 'Read more',
    ]);

    $this->get(route('landing'))
        ->assertSuccessful()
        ->assertInertia(fn ($page) => $page
            ->where('announcement.level', 'warning')
            ->where('announcement.title', 'Moving to a new address')
            ->where('announcement.message', 'This page moves to legacy.wormhole.systems on October 1st.')
            ->where('announcement.link.url', 'https://legacy.wormhole.systems')
            ->where('announcement.link.label', 'Read more')
            ->where('announcement.dismissible', true)
            ->has('announcement.id'));
});

it('hides an announcement the user dismissed', function () {
    setAnnouncementConfig(['announcement.message' => 'We are migrating.']);

    $this->withUnencryptedCookie('announcement_dismissed', Announcement::fromConfig()->id())
        ->get(route('landing'))
        ->assertSuccessful()
        ->assertInertia(fn ($page) => $page->where('announcement', null));
});

it('shows a new announcement even when an older one was dismissed', function () {
    setAnnouncementConfig(['announcement.message' => 'We are migrating.']);
    $dismissed = Announcement::fromConfig()->id();

    setAnnouncementConfig(['announcement.message' => 'The migration happens tonight.']);

    $this->withUnencryptedCookie('announcement_dismissed', $dismissed)
        ->get(route('landing'))
        ->assertSuccessful()
        ->assertInertia(fn ($page) => $page->where('announcement.message', 'The migration happens tonight.'));
});

it('ignores a dismissal for an announcement that cannot be dismissed', function () {
    setAnnouncementConfig([
        'announcement.message' => 'We are migrating.',
        'announcement.dismissible' => false,
    ]);

    $this->withUnencryptedCookie('announcement_dismissed', Announcement::fromConfig()->id())
        ->get(route('landing'))
        ->assertSuccessful()
        ->assertInertia(fn ($page) => $page->where('announcement.message', 'We are migrating.'));
});

it('falls back to the info level for an unknown level', function () {
    setAnnouncementConfig([
        'announcement.level' => 'nonsense',
        'announcement.message' => 'Scheduled maintenance tonight.',
    ]);

    expect(Announcement::fromConfig()->level)->toBe(AnnouncementLevel::Info);
});

it('labels the link with its url when no label is configured', function () {
    setAnnouncementConfig([
        'announcement.message' => 'We are migrating.',
        'announcement.link_url' => 'https://legacy.wormhole.systems',
    ]);

    expect(Announcement::fromConfig()->toArray()['link'])
        ->toBe(['url' => 'https://legacy.wormhole.systems', 'label' => 'https://legacy.wormhole.systems']);
});

it('omits the link when no url is configured', function () {
    setAnnouncementConfig([
        'announcement.message' => 'We are migrating.',
        'announcement.link_label' => 'Read more',
    ]);

    expect(Announcement::fromConfig()->toArray()['link'])->toBeNull();
});

it('can be marked as not dismissible', function () {
    setAnnouncementConfig([
        'announcement.message' => 'We are migrating.',
        'announcement.dismissible' => false,
    ]);

    expect(Announcement::fromConfig()->dismissible)->toBeFalse();
});

it('changes its id whenever the content changes', function () {
    setAnnouncementConfig(['announcement.message' => 'We are migrating.']);
    $first = Announcement::fromConfig()->id();

    setAnnouncementConfig(['announcement.message' => 'We are migrating on October 1st.']);
    $second = Announcement::fromConfig()->id();

    setAnnouncementConfig([
        'announcement.message' => 'We are migrating.',
        'announcement.level' => 'critical',
    ]);
    $third = Announcement::fromConfig()->id();

    expect($first)->not->toBe($second)
        ->and($first)->not->toBe($third);
});
