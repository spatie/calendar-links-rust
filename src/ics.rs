use crate::{Error, Generator, Link};
use base64::{Engine, engine::general_purpose::STANDARD};
use jiff::{
    Timestamp, ToSpan, Zoned,
    civil::DateTime,
    tz::{Dst, Offset, TimeZone},
};

/// Whether to return a data URI or a raw ICS file.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Presentation {
    /// A base64 data URI suitable for an HTML link.
    #[default]
    Html,
    /// The raw ICS body, including CRLF line endings.
    File,
}

/// A display alarm. The default is fifteen minutes before the event.
#[derive(Clone, Debug, Default)]
pub struct Reminder {
    /// Custom alarm description; defaults to `Reminder: <title>`.
    pub description: Option<String>,
    /// An absolute alarm time instead of the default relative trigger.
    pub time: Option<Timestamp>,
}

/// Optional RFC 5545 properties. Invalid tokens or control characters return an
/// error when the generator runs; dates and reminders are checked by Rust's types.
#[derive(Clone, Debug, Default)]
pub struct IcsOptions {
    /// Stable event identifier, defaulting to the PHP-compatible MD5 identifier.
    pub uid: Option<String>,
    /// Event URL, written verbatim as a URI.
    pub url: Option<String>,
    /// Product identifier. Defaults to `Spatie calendar-links`.
    pub prodid: Option<String>,
    /// Revision time, defaulting to the event start.
    pub dtstamp: Option<Timestamp>,
    /// An optional display alarm.
    pub reminder: Option<Reminder>,
    /// `OPAQUE` or `TRANSPARENT`, case insensitive.
    pub transparency: Option<String>,
    /// `PUBLIC`, `PRIVATE` or `CONFIDENTIAL`, case insensitive.
    pub class: Option<String>,
    /// Recurrence rule, written verbatim as a RECUR value.
    pub rrule: Option<String>,
    /// Microsoft busy status: `FREE`, `TENTATIVE`, `BUSY` or `OOF`.
    pub microsoft_busy_status: Option<String>,
}

/// Generates ICS data URIs and files, with optional properties and named timezones.
#[derive(Clone, Debug, Default)]
pub struct Ics {
    /// Event properties.
    pub options: IcsOptions,
    /// Output format.
    pub presentation: Presentation,
    /// Name every resolvable timed event's zones, unless an endpoint is repeated.
    pub with_timezones: bool,
    /// Extra calendar properties, already escaped, before nested components.
    pub calendar_properties: Vec<String>,
    /// Extra event properties, already escaped, before the alarm component.
    pub event_properties: Vec<String>,
}

impl Generator for Ics {
    fn generate(&self, link: &Link) -> Result<String, Error> {
        let options = &self.options;
        for (name, value) in [("URL", &options.url), ("RRULE", &options.rrule)] {
            if value
                .as_ref()
                .is_some_and(|s| s.chars().any(|c| c.is_ascii_control()))
            {
                return Err(Error::InvalidOption(name.to_owned()));
            }
        }
        let mut tokens = Vec::new();
        for (name, value, allowed) in [
            (
                "TRANSP",
                &options.transparency,
                &["OPAQUE", "TRANSPARENT"][..],
            ),
            (
                "CLASS",
                &options.class,
                &["PUBLIC", "PRIVATE", "CONFIDENTIAL"][..],
            ),
            (
                "X-MICROSOFT-CDO-BUSYSTATUS",
                &options.microsoft_busy_status,
                &["FREE", "TENTATIVE", "BUSY", "OOF"][..],
            ),
        ] {
            if let Some(value) = value {
                let value = value.to_ascii_uppercase();
                if !allowed.contains(&value.as_str()) {
                    return Err(Error::InvalidOption(name.to_owned()));
                }
                tokens.push(format!("{name}:{value}"));
            }
        }
        let mut lines = vec![
            "BEGIN:VCALENDAR".to_owned(),
            "VERSION:2.0".to_owned(),
            format!(
                "PRODID:{}",
                escape_text(options.prodid.as_deref().unwrap_or("Spatie calendar-links"))
            ),
        ];
        lines.extend(self.calendar_properties.iter().cloned());
        let named = self.names_timezones(link)?;
        let end_zone = if link.has_distinct_timezones() {
            link.to_timezone()
        } else {
            link.from_timezone()
        };
        if named {
            lines.extend(timezone_component(link.from_timezone(), link)?);
            if link.from_timezone() != end_zone {
                lines.extend(timezone_component(end_zone, link)?);
            }
        }
        let uid = options.uid.clone().unwrap_or_else(|| {
            format!(
                "{:x}",
                md5::compute(format!(
                    "{}{}{}{}",
                    link.from().strftime("%Y-%m-%dT%H:%M:%S%:z"),
                    link.to().strftime("%Y-%m-%dT%H:%M:%S%:z"),
                    link.title(),
                    link.event_address()
                ))
            )
        });
        lines.extend([
            "BEGIN:VEVENT".to_owned(),
            format!("UID:{}", escape_text(&uid)),
            format!("SUMMARY:{}", escape_text(link.title())),
            format!(
                "DTSTAMP:{}",
                options
                    .dtstamp
                    .unwrap_or_else(|| link.from().timestamp())
                    .strftime("%Y%m%dT%H%M%SZ")
            ),
        ]);
        if link.is_all_day() {
            lines.push(format!(
                "DTSTART;VALUE=DATE:{}",
                link.from().strftime("%Y%m%d")
            ));
            let days = link
                .to()
                .date()
                .since(link.from().date())?
                .get_days()
                .max(1);
            lines.push(format!("DURATION:P{days}D"));
        } else if named {
            lines.push(format!(
                "DTSTART;TZID={}:{}",
                link.from_timezone().iana_name().unwrap_or("UTC"),
                link.from().strftime("%Y%m%dT%H%M%S")
            ));
            lines.push(format!(
                "DTEND;TZID={}:{}",
                end_zone.iana_name().unwrap_or("UTC"),
                link.to()
                    .with_time_zone(end_zone.clone())
                    .strftime("%Y%m%dT%H%M%S")
            ));
        } else {
            lines.push(format!(
                "DTSTART:{}",
                link.from().timestamp().strftime("%Y%m%dT%H%M%SZ")
            ));
            lines.push(format!(
                "DTEND:{}",
                link.to().timestamp().strftime("%Y%m%dT%H%M%SZ")
            ));
        }
        if let Some(rule) = &options.rrule {
            lines.push(format!("RRULE:{rule}"));
        }
        if !link.event_description().is_empty() {
            lines.push(format!(
                "DESCRIPTION:{}",
                escape_text(&strip_tags(link.event_description()))
            ));
        }
        if !link.event_address().is_empty() {
            lines.push(format!("LOCATION:{}", escape_text(link.event_address())));
        }
        for guest in link.attendees() {
            let role = if guest.is_optional() {
                "OPT-PARTICIPANT"
            } else {
                "REQ-PARTICIPANT"
            };
            let address: String = guest
                .email()
                .chars()
                .map(|c| {
                    if c.is_ascii_control() || matches!(c, '%' | '&' | '?' | '=' | '/' | '#') {
                        format!("%{:02X}", u32::from(c))
                    } else {
                        c.to_string()
                    }
                })
                .collect();
            lines.push(format!("ATTENDEE;ROLE={role}:mailto:{address}"));
        }
        lines.extend(tokens);
        if let Some(url) = &options.url {
            lines.push(format!("URL;VALUE=URI:{url}"));
        }
        lines.extend(self.event_properties.iter().cloned());
        if let Some(reminder) = &options.reminder {
            lines.extend([
                "BEGIN:VALARM".to_owned(),
                "ACTION:DISPLAY".to_owned(),
                format!(
                    "DESCRIPTION:{}",
                    reminder.description.as_ref().map_or_else(
                        || format!("Reminder: {}", escape_text(link.title())),
                        |s| escape_text(s)
                    )
                ),
                reminder.time.map_or_else(
                    || "TRIGGER:-PT15M".to_owned(),
                    |t| format!("TRIGGER;VALUE=DATE-TIME:{}", t.strftime("%Y%m%dT%H%M%SZ")),
                ),
                "END:VALARM".to_owned(),
            ]);
        }
        lines.extend(["END:VEVENT".to_owned(), "END:VCALENDAR".to_owned()]);
        let body: String = lines.iter().map(|line| fold_line(line)).collect();
        Ok(match self.presentation {
            Presentation::File => body,
            Presentation::Html => format!(
                "data:text/calendar;charset=utf-8;base64,{}",
                STANDARD.encode(body)
            ),
        })
    }
}

impl Ics {
    fn names_timezones(&self, link: &Link) -> Result<bool, Error> {
        if link.is_all_day() || !link.has_resolvable_timezones() {
            return Ok(false);
        }
        if self.with_timezones {
            for endpoint in [
                link.from(),
                &link.to().with_time_zone(if link.has_distinct_timezones() {
                    link.to_timezone().clone()
                } else {
                    link.from_timezone().clone()
                }),
            ] {
                if endpoint
                    .time_zone()
                    .to_ambiguous_timestamp(endpoint.datetime())
                    .earlier()?
                    != endpoint.timestamp()
                {
                    return Ok(false);
                }
            }
            return Ok(true);
        }
        if link.has_distinct_timezones() {
            return Ok(true);
        }
        if self.options.rrule.is_none() {
            return Ok(false);
        }
        let start = shift_year(link.from(), -1)?;
        let end = shift_year(link.to(), 1)?.checked_add((6 * 366 * 24).hours())?;
        Ok(link
            .from_timezone()
            .following(start)
            .next()
            .is_some_and(|t| t.timestamp() < end))
    }
}

/// Escape an RFC 5545 TEXT value, including newlines and control characters.
pub fn escape_text(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace("\r\n", "\\n")
        .replace(['\r', '\n'], "\\n")
        .chars()
        .filter(|c| !c.is_ascii_control() || *c == '\t')
        .collect()
}

fn strip_tags(input: &str) -> String {
    // PHP removes markup but leaves its text and entities intact.
    let mut output = String::new();
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\0' {
            continue;
        }
        if c != '<' || chars.peek().is_some_and(|c| c.is_ascii_whitespace()) {
            output.push(c);
            continue;
        }
        let mut tag = String::new();
        let mut depth = 0;
        let mut quote = None;
        let processing = chars.peek() == Some(&'?');
        while let Some(c) = chars.next() {
            let previous = tag.chars().next_back();
            if tag.starts_with("!--") {
                if c == '>' && tag.ends_with("--") {
                    break;
                }
            } else if let Some(q) = quote {
                if c == q && (!processing || previous != Some('\\')) {
                    quote = None;
                }
            } else if c == '\'' || c == '"' {
                quote = Some(c);
            } else if c == '<' && !chars.peek().is_some_and(|c| c.is_ascii_whitespace()) {
                depth += 1;
            } else if c == '>' {
                if depth > 0 {
                    depth -= 1;
                } else if !processing || previous == Some('?') || tag.starts_with("?xml") {
                    break;
                }
            }
            tag.push(c);
        }
    }
    output
}

fn shift_year(date: &Zoned, years: i32) -> Result<Timestamp, Error> {
    let shifted = date.checked_add(years.years())?;
    // PHP carries February 29 into March, rather than clamping it to February 28.
    Ok(if date.month() == 2 && date.day() == 29 {
        shifted.checked_add(1.day())?
    } else {
        shifted
    }
    .timestamp())
}

fn fold_line(mut line: &str) -> String {
    let mut output = String::new();
    let mut budget = 75;
    while line.len() > budget {
        let mut end = budget;
        while !line.is_char_boundary(end) {
            end -= 1;
        }
        output.push_str(&line[..end]);
        output.push_str("\r\n ");
        line = &line[end..];
        budget = 74;
    }
    output.push_str(line);
    output.push_str("\r\n");
    output
}

#[derive(Debug)]
struct Onset {
    time: Timestamp,
    from: Offset,
    to: Offset,
    daylight: bool,
    abbreviation: String,
}

fn timezone_component(zone: &TimeZone, link: &Link) -> Result<Vec<String>, Error> {
    let start = shift_year(link.from(), -1)?;
    let end = shift_year(link.to(), 1)?;
    let probe = end.checked_add((6 * 366 * 24).hours())?;
    let info = zone.to_offset_info(start);
    let mut onsets = vec![Onset {
        time: start,
        from: info.offset(),
        to: info.offset(),
        daylight: info.dst() == Dst::Yes,
        abbreviation: info.abbreviation().to_owned(),
    }];
    let mut previous = info.offset();
    for transition in zone.following(start).take_while(|t| t.timestamp() < probe) {
        onsets.push(Onset {
            time: transition.timestamp(),
            from: previous,
            to: transition.offset(),
            daylight: transition.dst() == Dst::Yes,
            abbreviation: transition.abbreviation().to_owned(),
        });
        previous = transition.offset();
    }
    let mut lines = vec![
        "BEGIN:VTIMEZONE".to_owned(),
        format!("TZID:{}", zone.iana_name().unwrap_or("UTC")),
    ];
    for (index, onset) in onsets.iter().enumerate().take_while(|(_, o)| o.time < end) {
        let kind = if onset.daylight {
            "DAYLIGHT"
        } else {
            "STANDARD"
        };
        lines.extend([
            format!("BEGIN:{kind}"),
            format!(
                "DTSTART:{}",
                onset.from.to_datetime(onset.time).strftime("%Y%m%dT%H%M%S")
            ),
            format!("TZOFFSETFROM:{}", offset_string(onset.from)),
            format!("TZOFFSETTO:{}", offset_string(onset.to)),
        ]);
        if index > 0
            && !onsets[index + 1..]
                .iter()
                .any(|o| o.time < end && o.daylight == onset.daylight)
        {
            let candidates: Vec<_> = onsets[index..]
                .iter()
                .filter(|o| o.daylight == onset.daylight)
                .map(|o| o.from.to_datetime(o.time))
                .collect();
            if let Some(rule) = annual_rule(&candidates) {
                lines.push(rule);
            }
        }
        lines.extend([
            format!("TZNAME:{}", escape_text(&onset.abbreviation)),
            format!("END:{kind}"),
        ]);
    }
    lines.push("END:VTIMEZONE".to_owned());
    Ok(lines)
}

fn annual_rule(onsets: &[DateTime]) -> Option<String> {
    let first = onsets.first()?;
    if onsets.len() < 3
        || !onsets.iter().all(|o| {
            o.month() == first.month() && o.weekday() == first.weekday() && o.time() == first.time()
        })
    {
        return None;
    }
    let week = if onsets
        .iter()
        .all(|o| i16::from(o.day()) + 7 > i16::from(o.date().days_in_month()))
    {
        -1
    } else if onsets
        .iter()
        .all(|o| (o.day() - 1) / 7 == (first.day() - 1) / 7)
    {
        (first.day() - 1) / 7 + 1
    } else {
        return None;
    };
    Some(format!(
        "RRULE:FREQ=YEARLY;BYMONTH={};BYDAY={week}{}",
        first.month(),
        first.strftime("%a").to_string()[..2].to_ascii_uppercase()
    ))
}

fn offset_string(offset: Offset) -> String {
    let seconds = offset.seconds();
    let abs = seconds.unsigned_abs();
    let mut value = format!(
        "{}{:02}{:02}",
        if seconds < 0 { '-' } else { '+' },
        abs / 3600,
        abs % 3600 / 60
    );
    if !abs.is_multiple_of(60) {
        value.push_str(&format!("{:02}", abs % 60));
    }
    value
}
