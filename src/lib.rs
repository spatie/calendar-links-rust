#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

pub mod generators;
mod ics;

pub use ics::{Ics, IcsOptions, Presentation, Reminder, escape_text};
pub use jiff;
use jiff::{ToSpan, Zoned, civil::Date, tz::TimeZone};

/// An invalid event, option, or date calculation.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The event ends before it starts.
    #[error("The event must not end before it starts")]
    NegativeDateRange,
    /// An all-day event needs at least one day.
    #[error("An all-day event must last at least one day")]
    InvalidDayCount,
    /// A guest must have a plain, unquoted ASCII email address.
    #[error("Invalid guest email: {0}")]
    InvalidGuestEmail(String),
    /// An ICS option cannot be represented safely.
    #[error("Invalid ICS option: {0}")]
    InvalidOption(String),
    /// A date calculation exceeded the supported range.
    #[error(transparent)]
    Date(#[from] jiff::Error),
}

/// Generates a calendar link or file from an event.
pub trait Generator {
    /// Generate the output.
    ///
    /// # Errors
    /// Returns an error if options or dates cannot be represented.
    fn generate(&self, link: &Link) -> Result<String, Error>;
}

/// A guest invited to an event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Guest {
    email: String,
    optional: bool,
}

impl Guest {
    /// The address, in its original spelling.
    pub fn email(&self) -> &str {
        &self.email
    }
    /// Whether attendance is optional.
    pub fn is_optional(&self) -> bool {
        self.optional
    }
}

/// An event that can be added to a calendar.
#[derive(Clone, Debug)]
pub struct Link {
    title: String,
    from: Zoned,
    to: Zoned,
    to_timezone: TimeZone,
    all_day: bool,
    description: String,
    address: String,
    guests: Vec<Guest>,
}

impl Link {
    /// Create a timed event, keeping the original endpoint timezones.
    ///
    /// # Errors
    /// Returns [`Error::NegativeDateRange`] if `to` is before `from`.
    pub fn create(title: impl Into<String>, from: Zoned, to: Zoned) -> Result<Self, Error> {
        if from.timestamp() > to.timestamp() {
            return Err(Error::NegativeDateRange);
        }
        let to_timezone = to.time_zone().clone();
        let to = to.with_time_zone(from.time_zone().clone());
        Ok(Self {
            title: title.into(),
            from,
            to,
            to_timezone,
            all_day: false,
            description: String::new(),
            address: String::new(),
            guests: Vec::new(),
        })
    }

    /// Create an all-day event lasting `number_of_days` calendar days.
    ///
    /// # Errors
    /// Returns an error for zero days or dates outside the supported range.
    pub fn create_all_day(
        title: impl Into<String>,
        from: Zoned,
        number_of_days: u32,
    ) -> Result<Self, Error> {
        if number_of_days == 0 {
            return Err(Error::InvalidDayCount);
        }
        let to = from.checked_add(i64::from(number_of_days).days())?;
        let mut link = Self::create(title, from, to)?;
        link.all_day = true;
        Ok(link)
    }

    /// Create an all-day event from inclusive calendar dates.
    ///
    /// # Errors
    /// Returns an error for a reversed range or a date outside the supported range.
    pub fn all_day(
        title: impl Into<String>,
        from: Date,
        to: Date,
        timezone: TimeZone,
    ) -> Result<Self, Error> {
        if from > to {
            return Err(Error::NegativeDateRange);
        }
        let end = to.checked_add(1.day())?.to_zoned(timezone.clone())?;
        let mut link = Self::create(title, from.to_zoned(timezone)?, end)?;
        link.all_day = true;
        Ok(link)
    }

    /// Set the description of the event.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
    /// Set the address of the event.
    #[must_use]
    pub fn address(mut self, address: impl Into<String>) -> Self {
        self.address = address.into();
        self
    }
    /// Add a guest. Duplicate addresses keep their first spelling and role.
    ///
    /// # Errors
    /// Returns [`Error::InvalidGuestEmail`] for anything other than a plain email address.
    pub fn guest(mut self, email: impl Into<String>, optional: bool) -> Result<Self, Error> {
        let email = email.into();
        validate_email(&email)?;
        if !self
            .guests
            .iter()
            .any(|guest| guest.email.eq_ignore_ascii_case(&email))
        {
            self.guests.push(Guest { email, optional });
        }
        Ok(self)
    }
    /// Add several guests, ignoring duplicates.
    ///
    /// # Errors
    /// Returns [`Error::InvalidGuestEmail`] if an address is invalid.
    pub fn guests(
        mut self,
        emails: impl IntoIterator<Item = impl Into<String>>,
        optional: bool,
    ) -> Result<Self, Error> {
        for email in emails {
            self = self.guest(email, optional)?;
        }
        Ok(self)
    }
    /// The event title.
    pub fn title(&self) -> &str {
        &self.title
    }
    /// The start in its original timezone.
    pub fn from(&self) -> &Zoned {
        &self.from
    }
    /// The exclusive end, normalized into the start timezone.
    pub fn to(&self) -> &Zoned {
        &self.to
    }
    /// The original start timezone.
    pub fn from_timezone(&self) -> &TimeZone {
        self.from.time_zone()
    }
    /// The original end timezone.
    pub fn to_timezone(&self) -> &TimeZone {
        &self.to_timezone
    }
    /// Whether the event is all-day.
    pub fn is_all_day(&self) -> bool {
        self.all_day
    }
    /// The description.
    pub fn event_description(&self) -> &str {
        &self.description
    }
    /// The address.
    pub fn event_address(&self) -> &str {
        &self.address
    }
    /// The guests in insertion order.
    pub fn attendees(&self) -> &[Guest] {
        &self.guests
    }
    /// Whether the event crosses two genuinely different timezones.
    pub fn has_distinct_timezones(&self) -> bool {
        !self.all_day
            && self.from_timezone() != self.to_timezone()
            && (self.from.offset() != self.to_timezone.to_offset(self.to.timestamp())
                || (names_place(self.from_timezone()) && names_place(self.to_timezone())))
    }
    /// Whether calendar services can resolve every timezone the event names.
    pub fn has_resolvable_timezones(&self) -> bool {
        resolvable(self.from_timezone())
            && (!self.has_distinct_timezones() || resolvable(self.to_timezone()))
    }
    /// Render using any built-in or custom generator.
    ///
    /// # Errors
    /// Propagates errors from the generator.
    pub fn format_with(&self, generator: &(impl Generator + ?Sized)) -> Result<String, Error> {
        generator.generate(self)
    }
    /// A Google Calendar link.
    pub fn google(&self) -> String {
        generators::web(self, generators::Service::Google, &[], None)
    }
    /// A Yahoo Calendar link.
    pub fn yahoo(&self) -> String {
        generators::web(self, generators::Service::Yahoo, &[], None)
    }
    /// An outlook.live.com link.
    pub fn web_outlook(&self) -> String {
        generators::web(self, generators::Service::WebOutlook, &[], None)
    }
    /// An Office 365 link.
    pub fn web_office(&self) -> String {
        generators::web(self, generators::Service::WebOffice, &[], None)
    }
    /// A base64 data URI holding an ICS file.
    ///
    /// # Errors
    /// Returns an error if dates cannot be represented.
    pub fn ics(&self) -> Result<String, Error> {
        Ics::default().generate(self)
    }
    /// An ICS data URI retaining named timezones, except repeated instants.
    ///
    /// # Errors
    /// Returns an error if dates cannot be represented.
    pub fn ics_with_timezones(&self) -> Result<String, Error> {
        Ics {
            with_timezones: true,
            ..Ics::default()
        }
        .generate(self)
    }
}

fn names_place(zone: &TimeZone) -> bool {
    zone.iana_name().is_some_and(|name| {
        !name.starts_with("Etc/")
            && !matches!(
                name,
                "UTC"
                    | "Zulu"
                    | "Universal"
                    | "Greenwich"
                    | "GMT0"
                    | "GMT"
                    | "UCT"
                    | "CET"
                    | "EST"
                    | "MST"
                    | "HST"
                    | "MET"
                    | "WET"
                    | "EET"
            )
    })
}

fn resolvable(zone: &TimeZone) -> bool {
    zone.iana_name().is_some_and(|name| {
        name == "UTC"
            || (!name.starts_with("Etc/")
                && !matches!(
                    name,
                    "CET"
                        | "CST6CDT"
                        | "EET"
                        | "EST"
                        | "EST5EDT"
                        | "Factory"
                        | "GMT"
                        | "GMT+0"
                        | "GMT-0"
                        | "GMT0"
                        | "Greenwich"
                        | "HST"
                        | "MET"
                        | "MST"
                        | "MST7MDT"
                        | "PST8PDT"
                        | "UCT"
                        | "Universal"
                        | "WET"
                        | "Zulu"
                ))
    })
}

fn validate_email(email: &str) -> Result<(), Error> {
    let valid = email.is_ascii()
        && email.len() <= 254
        && email.rsplit_once('@').is_some_and(|(local, domain)| {
            !local.is_empty()
                && local.len() <= 64
                && !local.starts_with('.')
                && !local.ends_with('.')
                && !local.contains("..")
                && local
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b".!#$%&'*+-/=?^_`{|}~".contains(&b))
                && ((domain.contains('.')
                    && domain.rsplit('.').next().is_some_and(|label| {
                        label
                            .as_bytes()
                            .first()
                            .is_some_and(u8::is_ascii_alphabetic)
                    })
                    && domain.split('.').all(|label| {
                        !label.is_empty()
                            && label.len() <= 63
                            && !label.starts_with('-')
                            && !label.ends_with('-')
                            && label
                                .bytes()
                                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
                    }))
                    || domain
                        .strip_prefix('[')
                        .and_then(|s| s.strip_suffix(']'))
                        .is_some_and(|s| {
                            s.split_once(':')
                                .filter(|(prefix, _)| prefix.eq_ignore_ascii_case("IPv6"))
                                .map_or_else(
                                    || s.parse::<std::net::Ipv4Addr>().is_ok(),
                                    |(_, address)| address.parse::<std::net::Ipv6Addr>().is_ok(),
                                )
                        }))
        });
    if valid {
        Ok(())
    } else {
        Err(Error::InvalidGuestEmail(email.to_owned()))
    }
}

#[cfg(doctest)]
mod guide {
    include!(concat!(env!("OUT_DIR"), "/documentation.rs"));
}
