---
title: Usage
weight: 4
---

```rust
use spatie_calendar_links::{Ics, Link, Presentation};

let from = "2027-03-15T10:00[Europe/Brussels]".parse()?;
let to = "2027-03-15T17:00[Europe/Brussels]".parse()?;

let link = Link::create("Birthday party", from, to)?
    .description("A full day of hands-on testing. Bring a laptop.")
    .address("Samberstraat 69D, 2060 Antwerp, Belgium");

// A link that creates the event on Google calendar
println!("{}", link.google());

// A link that creates the event on Yahoo calendar
println!("{}", link.yahoo());

// A link that creates the event on outlook.live.com
println!("{}", link.web_outlook());

// A link that creates the event on outlook.cloud.microsoft
println!("{}", link.web_office());

// A data URI holding an ics file, for iCal, Apple Calendar and Outlook
println!("{}", link.ics()?);

// The raw ics body instead of a data URI, to attach to an email as a file
println!("{}", link.format_with(&Ics {
    presentation: Presentation::File,
    ..Ics::default()
})?);

// Any generator of your own
# struct YourGenerator;
# impl spatie_calendar_links::Generator for YourGenerator {
#     fn generate(&self, link: &Link) -> Result<String, spatie_calendar_links::Error> { Ok(link.google()) }
# }
println!("{}", link.format_with(&YourGenerator)?);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`google()` produces:

```text
https://calendar.google.com/calendar/render?action=TEMPLATE&dates=20270315T100000/20270315T170000&ctz=Europe/Brussels&text=Birthday+party&details=A+full+day+of+hands-on+testing.+Bring+a+laptop.&location=Samberstraat+69D%2C+2060+Antwerp%2C+Belgium
```

and `Presentation::File` produces:

```text
BEGIN:VCALENDAR
VERSION:2.0
PRODID:Spatie calendar-links
BEGIN:VEVENT
UID:dc01d073bb7e6c2bfae1bd5c43283062
SUMMARY:Birthday party
DTSTAMP:20270315T090000Z
DTSTART:20270315T090000Z
DTEND:20270315T160000Z
DESCRIPTION:A full day of hands-on testing. Bring a laptop.
LOCATION:Samberstraat 69D\, 2060 Antwerp\, Belgium
END:VEVENT
END:VCALENDAR
```

## All-day events

Use `create_all_day()` for a number of days, or `all_day()` for inclusive start and end dates. The stored end is exclusive, as calendar services expect.

```rust
# use spatie_calendar_links::{Link, jiff::{civil::date, tz::TimeZone}};
let link = Link::create_all_day(
    "Birthday",
    "2018-02-01T00:00[UTC]".parse()?,
    5,
)?;

let link = Link::all_day(
    "All day bugs",
    date(2024, 1, 25),
    date(2024, 1, 30),
    TimeZone::get("Pacific/Wake")?,
)?;
# assert!(link.google().contains("dates=20240125/20240131"));
# Ok::<(), Box<dyn std::error::Error>>(())
```

An all-day event has calendar dates, not instants. If your dates come from two zoned values, pass their `.date()` values so the end stays on the date the caller wrote, without gaining or losing a day through timezone conversion.

## Separate start and end timezones

Most events start and end in the same place. Some do not, and a flight is the obvious case. Pass a date in each zone and the difference is kept:

```rust
# use spatie_calendar_links::Link;
let flight = Link::create(
    "NH 106 Tokyo to Los Angeles",
    "2027-03-15T09:00[Asia/Tokyo]".parse()?,
    "2027-03-15T09:30[America/Los_Angeles]".parse()?,
)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Google names each end with `stz` and `etz`, which take priority over `ctz`, so `ctz` is left out:

```text
https://calendar.google.com/calendar/render?action=TEMPLATE&dates=20270315T090000/20270315T093000&stz=Asia/Tokyo&etz=America/Los_Angeles&text=NH+106+Tokyo+to+Los+Angeles
```

and the ics file names them with `TZID`, instead of flattening both ends to UTC:

```text
DTSTAMP:20270315T000000Z
DTSTART;TZID=Asia/Tokyo:20270315T090000
DTEND;TZID=America/Los_Angeles:20270315T093000
```

`DTSTAMP` stays in UTC, as RFC 5545 requires. Each zone named this way also gets its own `VTIMEZONE` component, placed before the event and holding the offsets in force around it, so the file stands on its own for a client that does not resolve bare IANA names (older Outlook desktop, for instance). The component lists every change the zone makes in the year either side of the event, and the last change of each kind carries a yearly recurrence rule whenever the zone repeats it on a fixed weekday of a fixed month, so the component keeps describing the zone for occurrences far beyond that year. A zone that moves its clocks on dates that do not repeat that way (Africa/Casablanca suspends its summer time for Ramadan and restores it weeks later) is written without a rule rather than with a guessed one, and there the listed year is all a client has to go on.

To retain named timezones even when both endpoints share a zone, use `ics_with_timezones()`:

```rust
# use spatie_calendar_links::{Ics, IcsOptions, Link, Presentation};
# let link = Link::create("Birthday party", "2027-03-15T10:00[Europe/Brussels]".parse()?, "2027-03-15T17:00[Europe/Brussels]".parse()?)?;
link.ics_with_timezones()?; // Base64 data URI
link.format_with(&Ics {
    options: IcsOptions { rrule: Some("FREQ=WEEKLY".into()), ..IcsOptions::default() },
    presentation: Presentation::File,
    with_timezones: true,
    ..Ics::default()
})?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

It names resolvable zones including `UTC` and regions without daylight saving such as `Asia/Tokyo`. All-day events remain date-only; other calendar generators are unaffected.

Both ends have to name a place for this. If either one does not, the pair is written in UTC together, since naming only one end would leave the other to be read in whichever zone the viewer sits in.

`ics_with_timezones()` has one additional fallback: when either endpoint is the second occurrence of a repeated local time after clocks move backward, both endpoints are written in UTC without `VTIMEZONE`. RFC 5545 interprets a local time with `TZID` as the first occurrence, so naming the zone there would change the supplied instant. This applies to half-hour and date-line rollbacks too. A recurrence using this fallback repeats in UTC, not local wall time.

`link.to()` is normalised into `link.from()`'s zone, so the two are directly comparable. The zones are recorded separately, on `link.from_timezone()` and `link.to_timezone()`.

## Guests

Every generator can invite guests, with a required or optional role:

```rust
# use spatie_calendar_links::Link;
# let link = Link::create("Birthday party", "2027-03-15T10:00[Europe/Brussels]".parse()?, "2027-03-15T17:00[Europe/Brussels]".parse()?)?;
let link = link.guest("freek@example.com", false)?
    .guests(["ruben@example.com", "alex@example.com"], false)?
    .guest("willem@example.com", true)?;
# assert_eq!(link.attendees().len(), 4);
# Ok::<(), Box<dyn std::error::Error>>(())
```

An address that is already on the list is ignored (compared without regard to case), so the first spelling and role you gave a guest are the ones that are kept.

Each service spells the same thing differently:

| Generator | Required | Optional |
| --- | --- | --- |
| Google | `add=` | `add=`, with an `_o` suffix on the address |
| WebOutlook / WebOffice | `to=` | `cc=` |
| Yahoo | `inv_list=` | `inv_list=` (Yahoo has no optional role, so optional guests are invited as required ones) |
| ICS | `ATTENDEE;ROLE=REQ-PARTICIPANT` | `ATTENDEE;ROLE=OPT-PARTICIPANT` |

So the ics file above gains:

```text
ATTENDEE;ROLE=REQ-PARTICIPANT:mailto:freek@example.com
ATTENDEE;ROLE=REQ-PARTICIPANT:mailto:ruben@example.com
ATTENDEE;ROLE=REQ-PARTICIPANT:mailto:alex@example.com
ATTENDEE;ROLE=OPT-PARTICIPANT:mailto:willem@example.com
```

Two things to keep in mind. Only plain email addresses are accepted, so a display name (the `Name <email>` form) returns `Error::InvalidGuestEmail`, because Yahoo cannot represent one. And guests are emitted in addition to any custom URL parameters, so if you also pass `add`, `to`, `cc` or `inv_list` by hand, the parameter appears twice and the service decides which one wins.

## Yahoo parameters

Use a `WebGenerator` to pass extra query parameters:

```rust
# use spatie_calendar_links::{Link, generators::{Service, WebGenerator}};
# let link = Link::create("Birthday party", "2027-03-15T10:00[Europe/Brussels]".parse()?, "2027-03-15T17:00[Europe/Brussels]".parse()?)?;
println!("{}", link.format_with(&WebGenerator::new(Service::Yahoo)
    .parameter("TYPE", Some("7")))?);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`TYPE` picks the event charm, as a zero based index into Yahoo's list of 17 charms (`0` is General, `7` is Birthday, `13` is Phone).

Yahoo has no timezone parameter, so a timed event is sent as floating local time: an event created at 09:00 is composed as 09:00, whatever the reader's timezone.

A multi-day all-day event is composed as a timed event that runs from midnight on the first day to midnight on the last one. The dates and the duration are right, only the all-day banner is missing. `DUR=allday` is the parameter that flips the all-day toggle, but Yahoo ignores `DUR` as soon as `ET` is present, and `DUR` has no multi-day all-day value, so a multi-day event has to send `ET` to keep its end date. Single day all-day events are unaffected, since they need no `ET` and are sent as `ST=<date>&DUR=allday`.

Google, WebOutlook and WebOffice accept extra parameters in the same way. Call `.parameter()` more than once with the same key for a repeated parameter. `None::<String>` or an empty string produces a bare flag; `Some("0")` writes `=0`.

## ICS options

`IcsOptions` holds properties that are written into the file as RFC 5545 values:

```rust
# use spatie_calendar_links::{Ics, IcsOptions, Link, Reminder};
# let link = Link::create("Birthday party", "2027-03-15T10:00[Europe/Brussels]".parse()?, "2027-03-15T17:00[Europe/Brussels]".parse()?)?;
println!("{}", link.format_with(&Ics {
    options: IcsOptions {
        uid: Some("workshop-2027-03-15".into()),
        url: Some("https://example.com/events/birthday-party".into()),
        rrule: Some("FREQ=WEEKLY;BYDAY=MO,WE,FR".into()),
        transparency: Some("TRANSPARENT".into()),
        class: Some("PRIVATE".into()),
        reminder: Some(Reminder {
            description: Some("The workshop starts in 15 minutes".into()),
            ..Reminder::default()
        }),
        ..IcsOptions::default()
    },
    ..Ics::default()
})?);
# Ok::<(), Box<dyn std::error::Error>>(())
```

| Option | What it does | Specification |
| --- | --- | --- |
| `uid` | Identifies the event, so a later file carrying the same value updates it instead of creating a second one | [section 3.8.4.7](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.4.7) |
| `url` | Points at a page about the event | [section 3.8.4.6](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.4.6) |
| `prodid` | Names the product that wrote the file. Defaults to `Spatie calendar-links` | [section 3.7.3](https://datatracker.ietf.org/doc/html/rfc5545#section-3.7.3) |
| `dtstamp` | When the event information was last revised. Takes a `jiff::Timestamp` and is written in UTC. Defaults to the event start | [section 3.8.7.2](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.7.2) |
| `reminder` | Adds an alarm. Pass `Reminder::default()` for 15 minutes before, or set `description` and `time` | [section 3.6.6](https://datatracker.ietf.org/doc/html/rfc5545#section-3.6.6) |
| `rrule` | Repeats the event | [section 3.8.5.3](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.5.3) |
| `transparency` | `OPAQUE` shows the time as busy, `TRANSPARENT` shows it as free | [section 3.8.2.7](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.2.7) |
| `class` | Visibility: `PUBLIC`, `PRIVATE` or `CONFIDENTIAL` | [section 3.8.1.3](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.1.3) |
| `microsoft_busy_status` | `FREE`, `TENTATIVE`, `BUSY` or `OOF`. `TRANSP` only tells busy from free, so Outlook needs this one to show tentative or out of office | MS-OXCICAL |

`dtstamp` defaults to the event start rather than the moment the file is written, so the same `Link` always produces the same bytes and the output stays cacheable. The cost is that a client cannot read it as a revision signal when the same `UID` is imported twice. Pass your own value when that matters:

```rust
# use spatie_calendar_links::{Ics, IcsOptions, Link, jiff::Timestamp};
# let link = Link::create("Birthday party", "2027-03-15T10:00[Europe/Brussels]".parse()?, "2027-03-15T17:00[Europe/Brussels]".parse()?)?;
println!("{}", link.format_with(&Ics {
    options: IcsOptions {
        uid: Some("workshop-2027-03-15".into()),
        dtstamp: Some(Timestamp::now()),
        ..IcsOptions::default()
    },
    ..Ics::default()
})?);
# Ok::<(), Box<dyn std::error::Error>>(())
```

The value is written as a UTC date-time, for all-day events too, since RFC 5545 does not allow a bare date here.

`url` and `rrule` are written into the file as they are given, so they must not contain a control character, a carriage return or a line feed included. `transparency`, `class` and `microsoft_busy_status` take one of the tokens listed above in any case, and are written upper-cased. `class` deliberately accepts only those three, not the `x-name` and `iana-token` values RFC 5545 also permits. A value that breaks either rule returns `Error::InvalidOption`, so route user supplied data through the event's own fields (the title argument of `Link::create()` and `Link::create_all_day()`, `description()` and `address()`), which are escaped for you, rather than through these options.

`Reminder` and `Timestamp` are typed, so a misspelled type fails to compile instead of handing you the default alarm. String options are checked when generating the output.

The `presentation` field controls presentation rather than content:

```rust
# use spatie_calendar_links::{Ics, Link, Presentation};
# let link = Link::create("Birthday party", "2027-03-15T10:00[Europe/Brussels]".parse()?, "2027-03-15T17:00[Europe/Brussels]".parse()?)?;
println!("{}", link.format_with(&Ics {
    presentation: Presentation::File,
    ..Ics::default()
})?); // the raw ics body, rather than a data URI
# Ok::<(), Box<dyn std::error::Error>>(())
```

`Presentation::Html` (the default) and `Presentation::File` are the only two formats.

## Recurring events and timezones

A recurrence repeats the local time of its start, so the zone that time is read in decides where every later occurrence lands. `ics()` names zones for events with distinct resolvable endpoint zones and recurrences in zones with clock changes. `ics_with_timezones()` also names shared and fixed-offset regions, with or without an `rrule`. A named zone keeps a weekly 09:00 at 09:00 across daylight saving changes:

```text
DTSTART;TZID=Europe/Warsaw:20260323T090000
DTEND;TZID=Europe/Warsaw:20260323T093000
RRULE:FREQ=WEEKLY
```

Recurring events with `with_timezones: true` in `UTC` and fixed-offset regions such as `Asia/Tokyo` use the same named-zone representation. Bare offsets, unsupported abbreviations, and second occurrences of repeated local times keep UTC endpoints. `UNTIL` remains a UTC date-time for a timed recurrence; any `BYHOUR`, `BYMINUTE`, or `BYDAY` fields are interpreted in the timezone of `DTSTART`.

## Extending a generator

The generators cover the options that at least two services share. Everything else is deliberately left to custom generators. The ICS generator has two property lists for exactly that:

```rust
use spatie_calendar_links::{Generator, Ics, Link, Error, escape_text};
# let link = Link::create("Birthday party", "2027-03-15T10:00[Europe/Brussels]".parse()?, "2027-03-15T17:00[Europe/Brussels]".parse()?)?;

struct MyIcs;

impl Generator for MyIcs {
    fn generate(&self, link: &Link) -> Result<String, Error> {
        Ics {
            event_properties: vec![
                format!("CATEGORIES:{}", escape_text("Workshops")),
                "STATUS:CONFIRMED".into(),
            ],
            ..Ics::default()
        }.generate(link)
    }
}

println!("{}", link.format_with(&MyIcs)?);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`calendar_properties` does the same at the `VCALENDAR` level (for example `X-WR-CALNAME`). Supply complete, already escaped property lines, not nested components; use `escape_text()` for TEXT values. These extension lists are trusted generator output, not a place for unvalidated user input.

`WebGenerator` can be pointed at another endpoint with its `base_url` field. Use `generators::encode()` for service-specific query values. A completely custom generator only needs to implement the `Generator` trait.
