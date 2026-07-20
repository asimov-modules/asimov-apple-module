// This is free and unencumbered software released into the public domain.

#![no_std]
#![forbid(unsafe_code)]

#[cfg(feature = "std")]
extern crate std;

pub use ::jq::*;

#[cfg(feature = "std")]
pub fn calendar() -> &'static JsonFilter {
    use std::sync::OnceLock;
    static ONCE: OnceLock<JsonFilter> = OnceLock::new();
    ONCE.get_or_init(|| include_str!("jq/calendar.jq").parse().unwrap())
}

#[cfg(not(feature = "std"))]
pub fn calendar() -> JsonFilter {
    include_str!("jq/calendar.jq").parse().unwrap()
}

#[cfg(feature = "std")]
pub fn contacts() -> &'static JsonFilter {
    use std::sync::OnceLock;
    static ONCE: OnceLock<JsonFilter> = OnceLock::new();
    ONCE.get_or_init(|| include_str!("jq/contacts.jq").parse().unwrap())
}

#[cfg(not(feature = "std"))]
pub fn contacts() -> JsonFilter {
    include_str!("jq/contacts.jq").parse().unwrap()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn calendar_filter_preserves_event_fields() {
        let output = super::calendar()
            .filter_json(json!({
                "@type": "Event",
                "@id": "urn:apple:calendar:event:ABC123",
                "name": "Team Standup",
                "startDate": "Wednesday, June 25, 2025 at 9:00:00 AM",
                "endDate": "Wednesday, June 25, 2025 at 9:30:00 AM",
                "isPartOf": "Work",
                "source": "apple-calendar"
            }))
            .unwrap();

        assert_eq!(output["@type"], "Event");
        assert_eq!(output["source"], "apple-calendar");
        assert_eq!(output["name"], "Team Standup");
    }

    #[test]
    fn contacts_filter_preserves_person_fields() {
        let output = super::contacts()
            .filter_json(json!({
                "@type": "Person",
                "@id": "urn:apple:contacts:person:ABC123:ABPerson",
                "name": "Jane Appleseed",
                "source": "apple-contacts",
                "givenName": "Jane",
                "telephone": [
                    {
                        "@type": "ContactPoint",
                        "name": "mobile",
                        "value": "+5511986606176"
                    }
                ]
            }))
            .unwrap();

        assert_eq!(output["@type"], "Person");
        assert_eq!(output["source"], "apple-contacts");
        assert_eq!(output["givenName"], "Jane");
        assert_eq!(output["telephone"][0]["@type"], "ContactPoint");
        assert_eq!(output["telephone"][0]["value"], "+5511986606176");
    }
}
