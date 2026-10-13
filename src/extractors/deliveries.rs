use sanitizer::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

use crate::models::deliveries;

/// Where a checkout says the order is going.
///
/// Every field is optional because a saved address can supply the location and
/// the client may send only an identifier. What is required is decided during
/// resolution, not here: a saved address still needs a recipient name and phone
/// inline, since `customer_address` carries neither.
#[derive(Deserialize, Serialize, Debug, Clone, Default, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CheckoutDelivery {
    #[sanitizer(trim)]
    #[validate(length(min = 1, message = "saved address identifier cannot be empty"))]
    pub saved_address_identifier: Option<String>,
    #[sanitizer(trim)]
    #[validate(length(max = 100, message = "recipient name is too long"))]
    pub recipient_name: Option<String>,
    #[sanitizer(trim)]
    #[validate(length(max = 50, message = "recipient phone number is too long"))]
    pub recipient_phone: Option<String>,
    #[sanitizer(trim)]
    #[validate(length(max = 255, message = "address line one is too long"))]
    pub address_line_one: Option<String>,
    #[sanitizer(trim)]
    #[validate(length(max = 255, message = "address line two is too long"))]
    pub address_line_two: Option<String>,
    #[sanitizer(trim)]
    #[validate(length(max = 120, message = "local government area is too long"))]
    pub local_govt_area: Option<String>,
    #[sanitizer(trim)]
    #[validate(length(max = 120, message = "city or town is too long"))]
    pub city_or_town: Option<String>,
    #[sanitizer(trim)]
    #[validate(length(max = 120, message = "state is too long"))]
    pub state: Option<String>,
    #[sanitizer(trim)]
    pub delivery_notes: Option<String>,
    #[serde(default)]
    pub save_to_address_book: bool,
}

#[derive(Debug, Clone)]
pub struct ResolvedDelivery {
    pub recipient_name: String,
    pub recipient_phone: String,
    pub address_line_one: String,
    pub address_line_two: Option<String>,
    pub local_govt_area: Option<String>,
    pub city_or_town: Option<String>,
    pub state: Option<String>,
    pub delivery_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DeliveryResponse {
    pub identifier: String,
    pub recipient_name: String,
    pub recipient_phone: String,
    pub address_line_one: String,
    pub address_line_two: Option<String>,
    pub local_govt_area: Option<String>,
    pub city_or_town: Option<String>,
    pub state: Option<String>,
    pub delivery_notes: Option<String>,
}

impl From<deliveries::Model> for DeliveryResponse {
    fn from(value: deliveries::Model) -> Self {
        Self {
            identifier: value.identifier,
            recipient_name: value.recipient_name,
            recipient_phone: value.recipient_phone,
            address_line_one: value.address_line_one,
            address_line_two: value.address_line_two,
            local_govt_area: value.local_govt_area,
            city_or_town: value.city_or_town,
            state: value.state,
            delivery_notes: value.delivery_notes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(payload: &str) -> CheckoutDelivery {
        serde_json::from_str(payload).unwrap()
    }

    #[test]
    fn reads_the_delivery_from_a_camel_case_payload() {
        let delivery = parsed(
            r#"{
                "recipientName":"Ada Lovelace",
                "recipientPhone":"08012345678",
                "addressLineOne":"1 Analytical Way",
                "addressLineTwo":"Marylebone",
                "localGovtArea":"Ikeja",
                "cityOrTown":"Lagos",
                "state":"Lagos",
                "deliveryNotes":"Ring the bell twice",
                "saveToAddressBook":true
            }"#,
        );

        assert_eq!(delivery.recipient_name.as_deref(), Some("Ada Lovelace"));
        assert_eq!(delivery.recipient_phone.as_deref(), Some("08012345678"));
        assert_eq!(
            delivery.address_line_one.as_deref(),
            Some("1 Analytical Way")
        );
        assert_eq!(delivery.address_line_two.as_deref(), Some("Marylebone"));
        assert_eq!(delivery.local_govt_area.as_deref(), Some("Ikeja"));
        assert_eq!(delivery.city_or_town.as_deref(), Some("Lagos"));
        assert_eq!(delivery.state.as_deref(), Some("Lagos"));
        assert_eq!(
            delivery.delivery_notes.as_deref(),
            Some("Ring the bell twice")
        );
        assert!(delivery.save_to_address_book);
    }

    /// Selecting a saved address is the common path, so it must not require the
    /// customer to resend the address by hand.
    #[test]
    fn accepts_a_saved_address_identifier_on_its_own() {
        let delivery = parsed(r#"{"savedAddressIdentifier":"addr-1"}"#);

        assert_eq!(delivery.saved_address_identifier.as_deref(), Some("addr-1"));
        assert!(delivery.recipient_phone.is_none());
        assert!(delivery.validate().is_ok());
    }

    #[test]
    fn save_to_address_book_defaults_to_false() {
        let delivery = parsed(r#"{"recipientPhone":"08012345678"}"#);

        assert!(!delivery.save_to_address_book);
    }

    /// Saving an address that was already in the book would create a duplicate
    /// of it, so the flag has to be a clear opt-in.
    #[test]
    fn save_to_address_book_is_off_unless_asked_for() {
        let delivery = parsed(
            r#"{"recipientPhone":"08012345678","addressLineOne":"1 Analytical Way","saveToAddressBook":false}"#,
        );

        assert!(!delivery.save_to_address_book);
    }

    #[test]
    fn every_field_is_optional_so_a_saved_address_can_supply_them() {
        let delivery = parsed("{}");

        assert!(delivery.saved_address_identifier.is_none());
        assert!(delivery.recipient_name.is_none());
        assert!(delivery.recipient_phone.is_none());
        assert!(delivery.address_line_one.is_none());
        assert!(delivery.validate().is_ok());
    }

    #[test]
    fn an_absent_address_is_not_the_same_as_a_blank_one() {
        let delivery = parsed(r#"{"recipientPhone":"   "}"#);
        let mut delivery = delivery;
        delivery.sanitize();

        // Blank after trimming, so resolution rejects it rather than persisting
        // a phone the courier cannot call.
        assert_eq!(delivery.recipient_phone.as_deref(), Some(""));
    }

    #[test]
    fn trims_the_whitespace_a_pasted_address_carries() {
        let mut delivery = parsed(
            r#"{
                "recipientName":"  Ada Lovelace  ",
                "recipientPhone":"  08012345678 ",
                "addressLineOne":"  1 Analytical Way  "
            }"#,
        );
        delivery.sanitize();

        assert_eq!(delivery.recipient_name.as_deref(), Some("Ada Lovelace"));
        assert_eq!(delivery.recipient_phone.as_deref(), Some("08012345678"));
        assert_eq!(
            delivery.address_line_one.as_deref(),
            Some("1 Analytical Way")
        );
    }

    /// A blank saved-address identifier is a mistake, not a request for the
    /// inline fields, so it fails rather than quietly falling through.
    #[test]
    fn rejects_a_blank_saved_address_identifier() {
        let mut delivery = parsed(r#"{"savedAddressIdentifier":"   "}"#);
        delivery.sanitize();

        assert!(delivery.validate().is_err());
    }

    #[test]
    fn rejects_fields_longer_than_the_columns_they_land_in() {
        let long = "x".repeat(300);

        let too_long_name = parsed(&format!(r#"{{"recipientName":"{long}"}}"#));
        let too_long_phone = parsed(&format!(r#"{{"recipientPhone":"{long}"}}"#));
        let too_long_line_one = parsed(&format!(r#"{{"addressLineOne":"{long}"}}"#));

        assert!(too_long_name.validate().is_err());
        assert!(too_long_phone.validate().is_err());
        assert!(too_long_line_one.validate().is_err());
    }
}
