//! Event construction, validation and custom generators.

use spatie_calendar_links::{
    Error, Generator, Ics, IcsOptions, Link, Presentation, Reminder, escape_text,
    generators::{Service, WebGenerator},
    jiff::{civil::date, tz::TimeZone},
};

fn event() -> Link {
    Link::create(
        "Birthday party",
        "2027-03-15T10:00[Europe/Brussels]".parse().unwrap(),
        "2027-03-15T17:00[Europe/Brussels]".parse().unwrap(),
    )
    .unwrap()
}

#[test]
fn rejects_reversed_ranges_and_zero_days() {
    assert!(matches!(
        Link::create("Meeting", event().to().clone(), event().from().clone()),
        Err(Error::NegativeDateRange)
    ));
    assert!(matches!(
        Link::create_all_day("Holiday", event().from().clone(), 0),
        Err(Error::InvalidDayCount)
    ));
    assert!(matches!(
        Link::all_day(
            "Holiday",
            date(2027, 3, 16),
            date(2027, 3, 15),
            TimeZone::UTC
        ),
        Err(Error::NegativeDateRange)
    ));
}

#[test]
fn equal_endpoints_are_allowed_for_timed_events() {
    let event = Link::create("Reminder", event().from().clone(), event().from().clone()).unwrap();
    assert!(
        event
            .google()
            .contains("dates=20270315T100000/20270315T100000")
    );
}

#[test]
fn all_day_dates_remain_inclusive_across_timezones_and_dst() {
    let event = Link::all_day(
        "Holiday",
        date(2026, 3, 28),
        date(2026, 3, 30),
        TimeZone::get("Europe/Brussels").unwrap(),
    )
    .unwrap();
    assert_eq!(event.to().date(), date(2026, 3, 31));
    assert!(event.google().contains("dates=20260328/20260331"));
    let body = event
        .format_with(&Ics {
            presentation: Presentation::File,
            ..Ics::default()
        })
        .unwrap();
    assert!(body.contains("DURATION:P3D\r\n"));
    assert!(!body.contains("TZID="));
}

#[test]
fn validates_plain_guest_addresses_and_keeps_first_role() {
    let link = event()
        .guest("Freek@example.com", false)
        .unwrap()
        .guests(["freek@example.com", "willem@example.com"], true)
        .unwrap();
    assert_eq!(link.attendees().len(), 2);
    assert!(!link.attendees()[0].is_optional());
    for invalid in [
        "",
        "a@localhost",
        "a@example.123",
        "a@example.123a",
        "a@[::1]",
        "Name <a@example.com>",
        "\"a,b\"@example.com",
        "a..b@example.com",
        "a@-example.com",
        "a@exämple.com",
        "a@example.com\r\nX:yes",
    ] {
        assert!(
            matches!(
                event().guest(invalid, false),
                Err(Error::InvalidGuestEmail(_))
            ),
            "{invalid:?}"
        );
    }
    for valid in [
        "a+b@example.com",
        "a@example.a123",
        "a@[127.0.0.1]",
        "a@[IPv6:::1]",
        "a@[iPv6:::1]",
        "who?what&why=how/now%then@example.com",
    ] {
        assert!(event().guest(valid, false).is_ok(), "{valid:?}");
    }
}

#[test]
fn rejects_unsafe_ics_options_and_unknown_tokens() {
    for bad in [
        "https://example.com\r\nX:injected",
        "https://example.com\0",
        "https://example.com\t",
    ] {
        for options in [
            IcsOptions {
                url: Some(bad.into()),
                ..IcsOptions::default()
            },
            IcsOptions {
                rrule: Some(bad.into()),
                ..IcsOptions::default()
            },
        ] {
            assert!(matches!(
                event().format_with(&Ics {
                    options,
                    ..Ics::default()
                }),
                Err(Error::InvalidOption(_))
            ));
        }
    }
    for options in [
        IcsOptions {
            transparency: Some("free".into()),
            ..IcsOptions::default()
        },
        IcsOptions {
            class: Some("X-CUSTOM".into()),
            ..IcsOptions::default()
        },
        IcsOptions {
            microsoft_busy_status: Some("away".into()),
            ..IcsOptions::default()
        },
    ] {
        assert!(matches!(
            event().format_with(&Ics {
                options,
                ..Ics::default()
            }),
            Err(Error::InvalidOption(_))
        ));
    }
}

#[test]
fn custom_generators_add_properties_before_nested_components() {
    struct MyIcs;
    impl Generator for MyIcs {
        fn generate(&self, link: &Link) -> Result<String, Error> {
            Ics {
                options: IcsOptions {
                    reminder: Some(Reminder::default()),
                    ..IcsOptions::default()
                },
                presentation: Presentation::File,
                calendar_properties: vec![format!("X-WR-CALNAME:{}", escape_text("Work; shops"))],
                event_properties: vec!["STATUS:CONFIRMED".into()],
                ..Ics::default()
            }
            .generate(link)
        }
    }
    let generator: &dyn Generator = &MyIcs;
    let body = event().format_with(generator).unwrap();
    assert!(body.contains("X-WR-CALNAME:Work\\; shops\r\nBEGIN:VEVENT"));
    assert!(body.contains("STATUS:CONFIRMED\r\nBEGIN:VALARM"));
    let web = WebGenerator {
        base_url: Some("https://example.com/?calendar".into()),
        ..WebGenerator::new(Service::Google)
    };
    assert!(
        event()
            .format_with(&web)
            .unwrap()
            .starts_with("https://example.com/?calendar&dates=")
    );
}

#[test]
fn convenience_methods_use_the_same_generators() {
    let link = event();
    for (service, actual) in [
        (Service::Google, link.google()),
        (Service::Yahoo, link.yahoo()),
        (Service::WebOutlook, link.web_outlook()),
        (Service::WebOffice, link.web_office()),
    ] {
        assert_eq!(
            actual,
            link.format_with(&WebGenerator::new(service)).unwrap()
        );
    }
    assert_eq!(
        link.ics().unwrap(),
        link.format_with(&Ics::default()).unwrap()
    );
    assert_eq!(
        link.ics_with_timezones().unwrap(),
        link.format_with(&Ics {
            with_timezones: true,
            ..Ics::default()
        })
        .unwrap()
    );
}
