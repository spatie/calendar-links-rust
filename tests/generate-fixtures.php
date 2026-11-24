<?php

// Regenerate with PHP 8.3+ against spatie/calendar-links a1590989934891fa79c145eb1108e42765218b17:
// php tests/generate-fixtures.php ../calendar-links > tests/fixtures/php.json
// Use a PHP build with the system timezone database at /usr/share/zoneinfo.
if (timezone_version_get() !== '0.system') {
    throw new RuntimeException('Fixture generation requires PHP with the system timezone database.');
}

spl_autoload_register(function ($class) use ($argv) {
    require $argv[1].'/src/'.str_replace('\\', '/', substr($class, strlen('Spatie\\CalendarLinks\\'))).'.php';
});

use Spatie\CalendarLinks\Link;

function datetime(string $value): DateTimeImmutable
{
    if (preg_match('/^(.+)\[(.+)]$/', $value, $matches)) {
        return new DateTimeImmutable($matches[1], new DateTimeZone($matches[2]));
    }

    return new DateTimeImmutable($value);
}

$cases = [];
$add = function (string $name, string $from, string $to, array $extra = []) use (&$cases) {
    $cases[] = array_merge(['name' => $name, 'title' => 'Birthday party', 'from' => $from, 'to' => $to], $extra);
};
$add('readme', '2027-03-15T10:00[Europe/Brussels]', '2027-03-15T17:00[Europe/Brussels]', ['description' => 'A full day of hands-on testing. Bring a laptop.', 'address' => 'Samberstraat 69D, 2060 Antwerp, Belgium']);
$add('flight', '2027-03-15T09:00[Asia/Tokyo]', '2027-03-15T09:30[America/Los_Angeles]', ['title' => 'NH 106 Tokyo to Los Angeles']);
$add('same offset places', '2026-01-15T09:00[Europe/London]', '2026-01-15T11:45[Europe/Lisbon]');
$add('alias places', '2026-01-15T09:00[Japan]', '2026-01-15T11:45[Asia/Seoul]');
$add('fixed regions', '2027-07-01T09:00[Asia/Kolkata]', '2027-07-01T15:30[Atlantic/Reykjavik]');
$add('unresolved flight', '2027-03-15T09:00[Asia/Tokyo]', '2027-03-15T09:30-07:00');
$add('utc aliases', '2026-01-01T10:00[UTC]', '2026-01-01T11:00[Etc/UTC]');
$add('zero fields', '2026-01-01T10:00[UTC]', '2026-01-01T11:00[UTC]', ['description' => '0', 'address' => '0']);
$add('escaping', '2026-01-01T10:00[UTC]', '2026-01-01T11:00[UTC]', ['title' => str_repeat('é😉', 24)."\\;,\r\n\0\x01\t", 'description' => '<p>Bring <b>a laptop</b> &amp; a dog.</p>'."\r\n", 'address' => 'A \\ B; C, D']);
$add('guests', '2026-01-01T10:00[UTC]', '2026-01-01T11:00[UTC]', ['guests' => [['freek@example.com', false], ['ruben@example.com', false], ['FREEK@example.com', true], ['willem@example.com', true], ['who?what&why=how/now%then@example.com', false]], 'parameters' => [['sprop', 'goo.allowModify:false'], ['sprop', 'goo.showInvitees:false'], ['flag', null], ['empty', ''], ['zero', '0'], ['tilde ~', '~ +'], ['inv_list', 'a+b@example.com,c@example.com']]]);
foreach (['UTC', 'Europe/Brussels', 'Europe/Warsaw', 'Asia/Tokyo', 'Africa/Casablanca', 'Australia/Lord_Howe', 'Pacific/Apia', 'America/Santiago', 'America/Sao_Paulo', 'Etc/GMT+5', 'EST', 'Factory', 'US/Pacific', 'GB'] as $zone) {
    $add($zone, "2026-06-01T09:00[$zone]", "2026-06-01T10:00[$zone]", ['options' => ['RRULE' => 'FREQ=WEEKLY;BYDAY=MO,WE,FR']]);
}
foreach (['+02:00', '-05:00'] as $offset) {
    $add($offset, "2026-06-01T09:00$offset", "2026-06-01T10:00$offset");
}
foreach ([1, 5, 400] as $days) {
    $add("all day $days", '2026-03-28T00:00[Europe/Brussels]', '2026-03-28T00:00[Europe/Brussels]', ['days' => $days]);
}
$add('all day offset', '2026-01-01T00:00+02:00', '2026-01-01T00:00+02:00', ['days' => 1]);
$add('second fold', '2026-10-25T02:15:00+01:00[Europe/Brussels]', '2026-10-25T03:30:00+01:00[Europe/Brussels]');
$add('first fold', '2026-10-25T02:15:00+02:00[Europe/Brussels]', '2026-10-25T03:30:00+01:00[Europe/Brussels]');
$add('half hour fold', '2026-04-05T01:45:00+10:30[Australia/Lord_Howe]', '2026-04-05T03:30:00+10:30[Australia/Lord_Howe]');
$add('options', '2027-03-15T10:00[Europe/Brussels]', '2027-03-15T17:00[Europe/Brussels]', ['options' => ['UID' => "workshop;\\,\n2027", 'PRODID' => 'My Product', 'URL' => 'https://example.com/events/birthday-party?a=1&b=2', 'DTSTAMP' => '2027-02-01T12:30:00Z', 'TRANSP' => 'transparent', 'CLASS' => 'private', 'X-MICROSOFT-CDO-BUSYSTATUS' => 'tentative', 'REMINDER' => ['DESCRIPTION' => "The workshop starts;\nBring a laptop.", 'TIME' => '2027-03-15T08:45:00Z']]]);
$add('default reminder', '2027-03-15T10:00[Europe/Brussels]', '2027-03-15T17:00[Europe/Brussels]', ['options' => ['REMINDER' => []]]);
$add('leap day', '2024-02-29T09:00[Europe/Brussels]', '2024-02-29T10:00[Europe/Brussels]', ['options' => ['RRULE' => 'FREQ=WEEKLY']]);
$add('dateline fold', '1969-09-30T10:00:00-12:00[Pacific/Kwajalein]', '1969-09-30T11:00:00-12:00[Pacific/Kwajalein]');
foreach (['1 < 2 and 3 > 2', 'a<!-- x > y -->b', 'a<?php echo "?>"; ?>b', '<p title="a > b">keep</p>', 'a<outer<inner>>b', '<script>keep text</script>', 'before<unfinished'] as $description) {
    $add("html $description", '2027-03-15T10:00[Europe/Brussels]', '2027-03-15T17:00[Europe/Brussels]', ['description' => $description]);
}

foreach ($cases as &$case) {
    foreach (['from', 'to'] as $key) {
        if (preg_match('/\[(.+)]$/', $case[$key], $matches)) {
            $destination = __DIR__.'/fixtures/zoneinfo/'.$matches[1];
            if (! is_dir(dirname($destination))) {
                mkdir(dirname($destination), 0777, true);
            }
            copy('/usr/share/zoneinfo/'.$matches[1], $destination);
        }
    }
    $from = datetime($case['from']);
    $to = datetime($case['to']);
    // An explicit offset disambiguates a fold; restore the named timezone afterwards.
    foreach (['from', 'to'] as $key) {
        if (preg_match('/^(.+[+-]\d\d:\d\d)\[(.+)]$/', $case[$key], $matches)) {
            $$key = (new DateTimeImmutable($matches[1]))->setTimezone(new DateTimeZone($matches[2]));
        }
    }
    $link = isset($case['days']) ? Link::createAllDay($case['title'], $from, $case['days']) : Link::create($case['title'], $from, $to);
    $link->description($case['description'] ?? '')->address($case['address'] ?? '');
    foreach ($case['guests'] ?? [] as [$email, $optional]) { $link->guest($email, $optional); }
    $parameters = [];
    foreach ($case['parameters'] ?? [] as [$key, $value]) { $parameters[$key][] = $value; }
    $options = $case['options'] ?? [];
    if (isset($options['DTSTAMP'])) { $options['DTSTAMP'] = datetime($options['DTSTAMP']); }
    if (isset($options['REMINDER']['TIME'])) { $options['REMINDER']['TIME'] = datetime($options['REMINDER']['TIME']); }
    $case['expected'] = [
        'google' => $link->google($parameters),
        'yahoo' => $link->yahoo($parameters),
        'web_outlook' => $link->webOutlook($parameters),
        'web_office' => $link->webOffice($parameters),
        'ics' => $link->ics($options, ['format' => 'file']),
        'ics_uri' => $link->ics($options),
        'ics_with_timezones' => $link->icsWithTimezones($options, ['format' => 'file']),
    ];
}
echo json_encode($cases, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE | JSON_THROW_ON_ERROR)."\n";
