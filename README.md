# spatie-calendar-links

[![Latest version on crates.io](https://img.shields.io/crates/v/spatie-calendar-links.svg?style=flat-square)](https://crates.io/crates/spatie-calendar-links)
[![Tests](https://img.shields.io/github/actions/workflow/status/spatie/calendar-links-rust/ci.yml?branch=main&style=flat-square)](https://github.com/spatie/calendar-links-rust/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](https://github.com/spatie/calendar-links-rust/blob/main/LICENSE.md)
[![Total downloads](https://img.shields.io/crates/d/spatie-calendar-links.svg?style=flat-square)](https://crates.io/crates/spatie-calendar-links)

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

Give both dates an explicit timezone, as above. The package can also generate ics files that you can open in several email and calendar programs, including Microsoft Outlook, Google Calendar, and Apple Calendar.

## Support us

[<img src="https://spatie.be/github-ad-image/calendar-links-rust.jpg" alt="Support Spatie's open source work" width="419px" />](https://spatie.be/github-ad-click/calendar-links-rust)

We invest a lot of resources into creating [best in class open source packages](https://spatie.be/open-source). You can support us by [buying one of our paid products](https://spatie.be/open-source/support-us).

We highly appreciate you sending us a postcard from your hometown, mentioning which of our package(s) you are using. You'll find our address on [our contact page](https://spatie.be/about-us). We publish all received postcards on [our virtual postcard wall](https://spatie.be/open-source/postcards).

## Installation

Requires Rust 1.89 or newer. Once the crate is published:

```sh
cargo add spatie-calendar-links
```

Before publication, use a path dependency from a neighboring application:

```toml
[dependencies]
spatie-calendar-links = { path = "../calendar-links-rust" }
```

## Usage

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

See the [documentation](https://spatie.be/docs/calendar-links-rust/v1/introduction) for all-day events, guests, separate start and end timezones, recurring events, ICS options and custom generators.

## Testing

```sh
cargo test --locked --all-targets --all-features
cargo test --locked --doc --all-features
```

See [all quality checks](https://github.com/spatie/calendar-links-rust/blob/main/docs/testing.md). The README and Markdown guides are tested by rustdoc, including setup on hidden `# ` lines.

## Changelog

See [CHANGELOG.md](https://github.com/spatie/calendar-links-rust/blob/main/CHANGELOG.md).

## Contributing

See [CONTRIBUTING.md](https://github.com/spatie/calendar-links-rust/blob/main/CONTRIBUTING.md) and the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).

Using an AI coding agent? Load Apollo's [rust-best-practices skill](https://github.com/apollographql/skills/tree/main/skills/rust-best-practices) when writing or reviewing code in this package.

## Security Vulnerabilities

Report vulnerabilities privately using the instructions in [SECURITY.md](https://github.com/spatie/calendar-links-rust/blob/main/SECURITY.md).

## Credits

- [Sebastian De Deyne](https://github.com/sebastiandedeyne)
- [Freek Van der Herten](https://github.com/freekmurze)
- [All contributors](https://github.com/spatie/calendar-links-rust/graphs/contributors)

## License

The MIT License (MIT). See [LICENSE.md](https://github.com/spatie/calendar-links-rust/blob/main/LICENSE.md).
