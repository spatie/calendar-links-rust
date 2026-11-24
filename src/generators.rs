//! Calendar service generators and custom URL parameters.

use crate::{Error, Generator, Link};
use jiff::tz::TimeZone;

/// An ordered list of extra query parameters. Repeat keys for multiple values;
/// `None` and empty strings produce a bare flag without an equals sign.
pub type UrlParameters = Vec<(String, Option<String>)>;

/// The calendar service a web generator targets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Service {
    /// Google Calendar.
    Google,
    /// Yahoo Calendar.
    Yahoo,
    /// Outlook on outlook.live.com.
    WebOutlook,
    /// Office 365 on outlook.cloud.microsoft.
    WebOffice,
}

/// A web calendar generator, optionally with extra parameters or a custom endpoint.
///
/// ```
/// # use spatie_calendar_links::{Link, generators::{WebGenerator, Service}};
/// # let link = Link::create("Birthday party", "2027-03-15T10:00[Europe/Brussels]".parse()?, "2027-03-15T17:00[Europe/Brussels]".parse()?)?;
/// let generator = WebGenerator::new(Service::Yahoo)
///     .parameter("TYPE", Some("7"));
/// let url = link.format_with(&generator)?;
/// # assert!(url.ends_with("&TYPE=7"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug)]
pub struct WebGenerator {
    /// The target calendar service.
    pub service: Service,
    /// Extra query parameters, appended in order.
    pub url_parameters: UrlParameters,
    /// Override the endpoint, including its initial query string.
    pub base_url: Option<String>,
}

impl WebGenerator {
    /// Create a generator for a calendar service.
    pub fn new(service: Service) -> Self {
        Self {
            service,
            url_parameters: Vec::new(),
            base_url: None,
        }
    }
    /// Append a parameter. Use `None` for a bare flag.
    #[must_use]
    pub fn parameter(mut self, key: impl Into<String>, value: Option<impl Into<String>>) -> Self {
        self.url_parameters
            .push((key.into(), value.map(Into::into)));
        self
    }
}

impl Generator for WebGenerator {
    fn generate(&self, link: &Link) -> Result<String, Error> {
        Ok(web(
            link,
            self.service,
            &self.url_parameters,
            self.base_url.as_deref(),
        ))
    }
}

/// Percent encode text as PHP's `rawurlencode` (or `urlencode` for Google).
pub fn encode(input: &str, form: bool) -> String {
    const HEX: &[u8] = b"0123456789ABCDEF";
    let mut output = String::new();
    for byte in input.bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.".contains(&byte) || (!form && byte == b'~') {
            output.push(char::from(byte));
        } else if form && byte == b' ' {
            output.push('+');
        } else {
            output.push('%');
            output.push(char::from(HEX[usize::from(byte >> 4)]));
            output.push(char::from(HEX[usize::from(byte & 15)]));
        }
    }
    output
}

pub(crate) fn web(
    link: &Link,
    service: Service,
    parameters: &[(String, Option<String>)],
    base: Option<&str>,
) -> String {
    let google = service == Service::Google;
    let yahoo = service == Service::Yahoo;
    let outlook = !google && !yahoo;
    let mut url = base.unwrap_or(match service {
        Service::Google => "https://calendar.google.com/calendar/render?action=TEMPLATE",
        Service::Yahoo => "https://calendar.yahoo.com/?v=60",
        Service::WebOutlook => "https://outlook.live.com/calendar/action/compose?path=/calendar/action/compose&rru=addevent",
        Service::WebOffice => "https://outlook.cloud.microsoft/calendar/deeplink/compose?path=/calendar/action/compose&rru=addevent",
    }).to_owned();
    let from = link.from();
    let to = link.to();
    if google {
        if link.is_all_day() {
            url.push_str(&format!(
                "&dates={}/{}",
                from.strftime("%Y%m%d"),
                to.strftime("%Y%m%d")
            ));
            if link.has_resolvable_timezones() {
                url.push_str(&format!(
                    "&ctz={}",
                    from.time_zone().iana_name().unwrap_or("UTC")
                ));
            }
        } else if !link.has_resolvable_timezones() {
            url.push_str(&format!(
                "&dates={}/{}",
                from.timestamp().strftime("%Y%m%dT%H%M%SZ"),
                to.timestamp().strftime("%Y%m%dT%H%M%SZ")
            ));
        } else if link.has_distinct_timezones() {
            url.push_str(&format!(
                "&dates={}/{}&stz={}&etz={}",
                from.strftime("%Y%m%dT%H%M%S"),
                to.with_time_zone(link.to_timezone().clone())
                    .strftime("%Y%m%dT%H%M%S"),
                from.time_zone().iana_name().unwrap_or("UTC"),
                link.to_timezone().iana_name().unwrap_or("UTC")
            ));
        } else {
            url.push_str(&format!(
                "&dates={}/{}&ctz={}",
                from.strftime("%Y%m%dT%H%M%S"),
                to.strftime("%Y%m%dT%H%M%S"),
                from.time_zone().iana_name().unwrap_or("UTC")
            ));
        }
    } else if yahoo {
        if link.is_all_day() {
            url.push_str(&format!("&ST={}", from.strftime("%Y%m%d")));
            if to
                .date()
                .since(from.date())
                .is_ok_and(|s| s.get_days() == 1)
            {
                url.push_str("&DUR=allday");
            } else {
                url.push_str(&format!("&ET={}", to.strftime("%Y%m%d")));
            }
        } else {
            url.push_str(&format!(
                "&ST={}&ET={}",
                from.strftime("%Y%m%dT%H%M%S"),
                to.strftime("%Y%m%dT%H%M%S")
            ));
        }
    } else if link.is_all_day() {
        url.push_str(&format!(
            "&startdt={}&enddt={}&allday=true",
            from.strftime("%Y-%m-%d"),
            to.strftime("%Y-%m-%d")
        ));
    } else {
        url.push_str(&format!(
            "&startdt={}&enddt={}",
            from.with_time_zone(TimeZone::UTC)
                .strftime("%Y-%m-%dT%H:%M:%SZ"),
            to.with_time_zone(TimeZone::UTC)
                .strftime("%Y-%m-%dT%H:%M:%SZ")
        ));
    }
    for (key, value) in [
        (
            if google {
                "text"
            } else if yahoo {
                "TITLE"
            } else {
                "subject"
            },
            link.title(),
        ),
        (
            if google {
                "details"
            } else if yahoo {
                "DESC"
            } else {
                "body"
            },
            link.event_description(),
        ),
        (
            if yahoo { "in_loc" } else { "location" },
            link.event_address(),
        ),
    ] {
        if !value.is_empty() || key == "text" || key == "TITLE" || key == "subject" {
            url.push_str(&format!("&{key}={}", encode(value, google)));
        }
    }
    if !link.attendees().is_empty() {
        if outlook {
            for (optional, key) in [(false, "to"), (true, "cc")] {
                let guests: Vec<_> = link
                    .attendees()
                    .iter()
                    .filter(|g| g.is_optional() == optional)
                    .map(|g| encode(g.email(), false))
                    .collect();
                if !guests.is_empty() {
                    url.push_str(&format!("&{key}={}", guests.join(",")));
                }
            }
        } else if yahoo {
            url.push_str(&format!(
                "&inv_list={}",
                link.attendees()
                    .iter()
                    .map(|g| encode(g.email(), false))
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        } else {
            let guests = link
                .attendees()
                .iter()
                .map(|g| format!("{}{}", g.email(), if g.is_optional() { "_o" } else { "" }))
                .collect::<Vec<_>>()
                .join(",");
            url.push_str(&format!("&add={}", encode(&guests, true)));
        }
    }
    for (key, value) in parameters {
        url.push('&');
        url.push_str(&encode(key, true));
        if let Some(value) = value.as_deref().filter(|s| !s.is_empty()) {
            url.push('=');
            if yahoo && key == "inv_list" {
                url.push_str(
                    &value
                        .split(',')
                        .map(|s| encode(s, false))
                        .collect::<Vec<_>>()
                        .join(","),
                );
            } else {
                url.push_str(&encode(value, google));
            }
        }
    }
    url
}
