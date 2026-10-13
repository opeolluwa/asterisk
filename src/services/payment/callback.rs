//! Where a customer is sent once they have paid.

use crate::utils::url::encode_path_segment;

/// The storefront page that confirms a payment.
///
/// The path lives here rather than at the call sites so the REST checkout, the
/// GraphQL checkout and the webhook cannot drift onto different URLs. The
/// storefront owns what this path renders; this crate owns only the address.
const SUCCESS_PATH: &str = "/account/orders/success";

/// Builds the return URL for a checkout covering `order_identifiers`.
///
/// The order identifier is carried as a query parameter only when exactly one
/// order is being paid for, because a single `?order=` cannot name several. A
/// multi-order charge returns to the bare confirmation page and leaves the
/// storefront to name the order from what it recorded before leaving.
///
/// `frontend_url` is trimmed of trailing slashes so `https://example.com/` and
/// `https://example.com` produce the same URL.
pub fn success_callback_url(frontend_url: &str, order_identifiers: &[String]) -> String {
    let base = format!(
        "{}{SUCCESS_PATH}",
        frontend_url.trim().trim_end_matches('/')
    );

    match order_identifiers {
        [only] => format!("{base}?order={}", encode_path_segment(only)),
        _ => base,
    }
}

#[cfg(test)]
mod tests {
    use super::success_callback_url;

    fn identifiers(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| (*v).to_string()).collect()
    }

    #[test]
    fn carries_the_order_when_exactly_one_is_paid_for() {
        let url = success_callback_url("https://asterisk.app", &identifiers(&["01HQZX"]));

        assert_eq!(
            url,
            "https://asterisk.app/account/orders/success?order=01HQZX"
        );
    }

    #[test]
    fn omits_the_order_when_several_are_paid_for() {
        // One query key cannot name two orders, and naming only the first would
        // send the customer to a confirmation about a partial payment.
        let url = success_callback_url(
            "https://asterisk.app",
            &identifiers(&["01HQZX", "01HQZY"]),
        );

        assert_eq!(url, "https://asterisk.app/account/orders/success");
    }

    #[test]
    fn omits_the_order_when_there_are_none() {
        let url = success_callback_url("https://asterisk.app", &[]);

        assert_eq!(url, "https://asterisk.app/account/orders/success");
    }

    #[test]
    fn does_not_double_the_slash_when_configured_with_a_trailing_one() {
        let url = success_callback_url("https://asterisk.app/", &[]);

        assert_eq!(url, "https://asterisk.app/account/orders/success");
    }

    #[test]
    fn escapes_an_identifier_rather_than_trusting_it() {
        let url = success_callback_url(
            "https://asterisk.app",
            &identifiers(&["a&order=victim"]),
        );

        assert_eq!(
            url,
            "https://asterisk.app/account/orders/success?order=a%26order%3Dvictim"
        );
    }
}
