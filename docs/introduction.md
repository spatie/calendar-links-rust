---
title: Introduction
weight: 1
---

Using this package, you can generate links to add events to calendar systems. Here's a quick example:

```rust
use spatie_calendar_links::Link;

Link::create(
    "Birthday party",
    "2027-03-15T10:00[Europe/Brussels]".parse()?,
    "2027-03-15T17:00[Europe/Brussels]".parse()?,
)?.google();
# Ok::<(), Box<dyn std::error::Error>>(())
```

This will output:

```text
https://calendar.google.com/calendar/render?action=TEMPLATE&dates=20270315T100000/20270315T170000&ctz=Europe/Brussels&text=Birthday+party
```

Give both dates an explicit timezone, as above. The `ctz` parameter is taken from the start date.

Name a place (`Europe/Brussels`, or an alias of one such as `Japan` or `US/Pacific`) rather than an offset. Neither an offset nor an abbreviation gives a calendar service a place to resolve. The same goes for the IANA entries that stand for no place: the POSIX rule sets (`EST`, `MST7MDT`), the other spellings of UTC, and the whole `Etc/GMT±N` family, which Google rejects outright and whose sign runs the opposite way from the offset it names. Those events are written in UTC instead (`dates=20270315T090000Z/20270315T160000Z`, with no `ctz`, and a UTC `DTSTART` in the ics file), so they land at the right instant everywhere, but the calendar has no zone to follow when the daylight saving rules of that place change. The `Etc/` tree is refused in full, `Etc/UTC` included, so pass `UTC` when you want a UTC endpoint alongside a named zone, since a refused zone at either end of a two zone event sends both ends to the fallback.

Dates use [`jiff::Zoned`](https://docs.rs/jiff/0.2/jiff/struct.Zoned.html), re-exported through `spatie_calendar_links::jiff`. A named date uses the `2027-03-15T10:00[Europe/Brussels]` syntax; a bare offset uses `2027-03-15T10:00+01:00[+01:00]`. Use an explicit offset for an abbreviation such as `CEST`, rather than relying on a date parser to guess its meaning.

If you follow that link (and are authenticated with Google), you'll see a screen to add the event to your calendar.

The package can also generate ics files that you can open in several email and calendar programs, including Microsoft Outlook, Google Calendar, and Apple Calendar.

## Package principles

1. it should produce a small output (to keep page-size small)
2. it should be fast (no external heavy dependencies)
3. all `Link` features should be supported by at least 2 generators (different services have different features)
