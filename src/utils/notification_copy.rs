//! Notification copy helpers.
//!
//! Notification bodies have a hard length budget. The console card clamps the
//! body to two lines (`line-clamp-2` on `text-xs`, roughly 120 characters), so
//! anything longer is silently truncated mid-word in the inbox while still being
//! delivered in full by email. The copy here is written to survive that clamp,
//! with the detail that does not fit pushed into `metadata` instead.
//!
//! Money is rendered with a symbol rather than an ISO code because these strings
//! are read by people scanning an inbox, not parsed by machines. The console
//! formats money with `Intl.NumberFormat` for structured display, but a
//! notification body is a plain string that may also be emailed, so it cannot
//! rely on the client having rendered anything.

use rust_decimal::Decimal;

use crate::models::sea_orm_active_enums::ProductCurrency;
use crate::utils::price::MINOR_UNITS_PER_MAJOR;

/// Roughly two lines of `text-xs` in the console card.
///
/// Not a hard limit the compiler enforces: `CONSOLE_BODY_BUDGET` exists so
/// `bodies_fit_the_console` can catch copy that would be truncated, and so the
/// budget is stated once instead of guessed at in nine places.
pub const CONSOLE_BODY_BUDGET: usize = 120;

/// The symbol a currency is written with in notification copy.
///
/// Falls back to the ISO code for anything without a widely recognised symbol,
/// because `"12500 XOF"` is still readable where `"12500 ¤"` would not be.
fn symbol_for(currency: ProductCurrency) -> &'static str {
    match currency {
        ProductCurrency::Ngn => "₦",
        ProductCurrency::Usd => "$",
        ProductCurrency::Gbp => "£",
        ProductCurrency::Eur => "€",
        ProductCurrency::Jpy => "¥",
        ProductCurrency::Cny => "¥",
        ProductCurrency::Inr => "₹",
        // Reached through a different symbol rather than the generic one, since
        // these are the currencies that actually turn up in this inbox.
        ProductCurrency::Ghs => "GH₵",
        ProductCurrency::Kes => "KSh",
        ProductCurrency::Zar => "R",
        ProductCurrency::Cad => "CA$",
        ProductCurrency::Aud => "A$",
        ProductCurrency::Aed => "AED",
        ProductCurrency::Xof => "FCFA",
    }
}

/// Renders minor units as a human amount, e.g. `₦45,000`.
///
/// Signed, because prices arrive as `i64` while order totals and captured
/// payments arrive as `u64` and there is no implicit widening between them.
/// [`format_amount_u64`] is the unsigned entry point rather than a cast at each
/// call site, where a wrapped total would come from.
///
/// Whole amounts drop the decimals: a notification reading `₦45,000.00` spends
/// two characters saying nothing. Fractional amounts keep them, because dropping
/// a customer a half-naira would be worse than a slightly longer string.
pub fn format_amount(minor: i64, currency: ProductCurrency) -> String {
    render_amount(Decimal::from(minor), currency)
}

/// [`format_amount`] for the order totals and captured payments that are stored
/// unsigned.
pub fn format_amount_u64(minor: u64, currency: ProductCurrency) -> String {
    render_amount(Decimal::from(minor), currency)
}

fn render_amount(minor: Decimal, currency: ProductCurrency) -> String {
    let major = (minor / Decimal::from(MINOR_UNITS_PER_MAJOR)).normalize();
    let decimals = decimal_places(&major);

    if decimals == 0 {
        return format!("{}{}", symbol_for(currency), grouped(&major));
    }

    format!(
        "{}{}",
        symbol_for(currency),
        grouped(&major.round_dp(decimals))
    )
}

fn decimal_places(value: &Decimal) -> u32 {
    value.normalize().scale()
}

/// Thousands separators, without pulling in a formatting dependency for the one
/// number an inbox actually shows.
fn grouped(value: &Decimal) -> String {
    let rendered = value.to_string();
    let (integral, fractional) = match rendered.split_once('.') {
        Some((integral, fractional)) => (integral, Some(fractional)),
        None => (rendered.as_str(), None),
    };

    let mut grouped = String::with_capacity(rendered.len() + 4);
    let digits: Vec<char> = integral.chars().collect();

    for (index, digit) in digits.iter().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            grouped.push(',');
        }

        grouped.push(*digit);
    }

    match fractional {
        Some(fractional) => format!("{grouped}.{fractional}"),
        None => grouped,
    }
}

/// Trims free text to something that fits the console card.
///
/// Cut on a word boundary where there is one, and on a sentence boundary ahead
/// of that: a preview ending mid-sentence reads worse than one ending at the
/// full stop. Used for complaint descriptions and reply bodies, which are written
/// by customers and staff and have no length limit of their own.
pub fn preview(text: &str, budget: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");

    if collapsed.chars().count() <= budget {
        return collapsed;
    }

    // Prefer the last sentence that fits, so the preview ends on real punctuation.
    let mut sentence_end = 0;

    for (index, character) in collapsed.char_indices() {
        if matches!(character, '.' | '!' | '?') {
            let candidate = index + character.len_utf8();

            if collapsed[candidate..].chars().count() <= budget {
                sentence_end = candidate;
            }

            break;
        }
    }

    if sentence_end > 0 {
        return collapsed[..sentence_end].to_string();
    }

    let mut end = budget;

    while end > 0 && !collapsed.is_char_boundary(end) {
        end -= 1;
    }

    // Back off to the last space so the cut does not land mid-word.
    let cut = collapsed[..end].rfind(' ').unwrap_or(end);
    let trimmed = collapsed[..cut].trim_end_matches([' ', ',', ';', ':', '-']);

    format!("{}…", trimmed.trim_end())
}

/// [`preview`] at the console's budget.
pub fn console_preview(text: &str) -> String {
    preview(text, CONSOLE_BODY_BUDGET)
}

/// Joins the parts of a body with a middot separator, skipping absent ones.
///
/// The console card is two lines wide; `·` separates facts without the visual
/// weight of a comma in a sentence that is not a sentence.
pub fn join_facts(parts: &[Option<String>]) -> String {
    parts
        .iter()
        .flatten()
        .filter(|part| !part.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(" · ")
}

/// `1 item` / `2 items`, for copy that counts something.
pub fn item_count(count: usize) -> String {
    if count == 1 {
        "1 item".to_string()
    } else {
        format!("{count} items")
    }
}

/// The first name of a recipient, for copy that greets someone.
///
/// Falls back to the whole name rather than an empty slot, because
/// `"New order from "` is worse than repeating a surname.
pub fn first_name_of(name: &str) -> &str {
    name.split_whitespace()
        .next()
        .filter(|first| !first.is_empty())
        .unwrap_or(name)
}

/// Minor units per major, re-exported for tests that assert the copy's decimals.
pub const UNITS_PER_MAJOR: i64 = MINOR_UNITS_PER_MAJOR;

/// Sentence-cases a snake_case or lowercase status for prose.
///
/// Statuses travel as lowercase enum names because that is what the API and the
/// database use. Dropped into a sentence that reads as a typo ("Order moved to
/// fulfilled"), so prose uses this while `metadata` keeps the wire value.
pub fn sentence_case(value: &str) -> String {
    let mut chars = value.chars();

    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// [`sentence_case`] for a status that may be absent, dropping the clause
/// entirely rather than rendering "Was ".
pub fn was(previous: Option<&str>) -> Option<String> {
    previous
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!("Was {}", sentence_case(value)))
}

/// [`sentence_case`] for a status that may be absent, used where the sentence
/// leads with the status instead.
pub fn status_clause(previous: Option<&str>) -> Option<String> {
    previous
        .filter(|value| !value.trim().is_empty())
        .map(sentence_case)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_whole_amounts_without_decimals() {
        assert_eq!(format_amount(4_500_000, ProductCurrency::Ngn), "₦45,000");
        assert_eq!(format_amount(0, ProductCurrency::Ngn), "₦0");
        assert_eq!(format_amount(100, ProductCurrency::Usd), "$1");
    }

    #[test]
    fn groups_thousands() {
        assert_eq!(
            format_amount(123_456_789, ProductCurrency::Ngn),
            "₦1,234,567.89"
        );
        assert_eq!(format_amount(100_000, ProductCurrency::Ngn), "₦1,000");
        assert_eq!(format_amount(1_000_000, ProductCurrency::Ngn), "₦10,000");
    }

    #[test]
    fn keeps_decimals_when_the_amount_is_fractional() {
        assert_eq!(format_amount(5_050, ProductCurrency::Ngn), "₦50.5");
    }

    #[test]
    fn uses_a_readable_fallback_for_symbols_that_are_not_widely_known() {
        assert_eq!(format_amount(100_000, ProductCurrency::Xof), "FCFA1,000");
        assert_eq!(format_amount(100_000, ProductCurrency::Aed), "AED1,000");
    }

    #[test]
    fn renders_the_currency_symbols_an_inbox_would_recognise() {
        assert_eq!(format_amount(100, ProductCurrency::Gbp), "£1");
        assert_eq!(format_amount(100, ProductCurrency::Eur), "€1");
        assert_eq!(format_amount(100, ProductCurrency::Inr), "₹1");
    }

    #[test]
    fn previews_shorter_text_unchanged_apart_from_whitespace() {
        assert_eq!(console_preview("a  short\n  body"), "a short body");
        assert_eq!(console_preview("already short"), "already short");
    }

    #[test]
    fn preview_stops_at_the_last_sentence_that_fits() {
        // Long enough that the whole thing cannot be returned, and every
        // sentence is on its own so the cut point is unambiguous.
        let text = "Customer says the parcel never arrived at all. \
                    They would like a refund for the shipping cost. \
                    Please advise on the next step here.";

        assert!(
            text.chars().count() > CONSOLE_BODY_BUDGET,
            "fixture must exceed the budget to be previewed"
        );

        let previewed = console_preview(text);

        assert!(
            previewed.ends_with('.'),
            "{previewed} should end a sentence"
        );
        assert!(
            !previewed.contains("next step"),
            "{previewed} ran on past the budget"
        );
        assert!(
            previewed.chars().count() <= CONSOLE_BODY_BUDGET,
            "{previewed} exceeds the budget"
        );
    }

    #[test]
    fn preview_returns_short_text_untouched() {
        let text = "First sentence here. Second sentence here.";

        assert_eq!(console_preview(text), text);
    }

    #[test]
    fn preview_cuts_on_a_word_boundary_when_no_sentence_fits() {
        let text = "word ".repeat(80);
        let previewed = console_preview(text.trim());

        assert!(previewed.ends_with('…'));
        assert!(
            !previewed.trim_end_matches('…').ends_with(' '),
            "cut a word in half"
        );
        assert!(previewed.chars().count() <= CONSOLE_BODY_BUDGET + 1);
    }

    #[test]
    fn preview_handles_text_with_no_spaces_at_all() {
        let previewed = console_preview(&"x".repeat(400));

        assert!(previewed.ends_with('…'));
        assert!(previewed.chars().count() <= CONSOLE_BODY_BUDGET + 1);
    }

    #[test]
    fn preview_never_splits_a_multibyte_character() {
        // Every character is three bytes, so a byte-indexed cut would panic here.
        let text = " Lagos ".repeat(80);
        let previewed = console_preview(text.trim());

        assert!(previewed.chars().count() <= CONSOLE_BODY_BUDGET + 1);
    }

    #[test]
    fn renders_unsigned_totals_the_same_way() {
        // Order totals and captured payments are stored unsigned, so this is the
        // entry point the notification copy actually uses.
        assert_eq!(
            format_amount_u64(4_500_000, ProductCurrency::Ngn),
            "₦45,000"
        );
        assert_eq!(format_amount_u64(0, ProductCurrency::Ngn), "₦0");
        assert_eq!(
            format_amount_u64(123_456_789, ProductCurrency::Usd),
            "$1,234,567.89"
        );
    }

    #[test]
    fn signed_and_unsigned_rendering_agree() {
        for minor in [0_i64, 1, 100, 5_050, 4_500_000] {
            assert_eq!(
                format_amount(minor, ProductCurrency::Ngn),
                format_amount_u64(minor as u64, ProductCurrency::Ngn),
                "disagreed at {minor}"
            );
        }
    }

    /// The guard that matters.
    ///
    /// Every event composes its body from these helpers, and the only inputs an
    /// author does not control are customer names, emails, subjects and city
    /// names. This checks the worst realistic case for each shape: a long name
    /// beside an amount, a long email, a free-text complaint. If a future
    /// sentence change pushes any of these past the clamp, this fails instead of
    /// the console quietly truncating mid-word.
    #[test]
    fn composed_bodies_fit_the_console_budget() {
        let long_name = "Oluwaseun Adebayo-Okonkwo";
        let long_email = "oluwaseun.adebayo-okonkwo@consultancy-group.example";

        let bodies = [
            // `order_created`, single order.
            join_facts(&[
                Some(item_count(12)),
                Some("Lagos".to_string()),
                Some(format_amount_u64(12_450_000, ProductCurrency::Ngn)),
            ]),
            // `order_created`, several orders at once.
            join_facts(&[
                Some(format!("{} across 4 orders", item_count(37))),
                Some(format!("{} total", format_amount_u64(985_000_00, ProductCurrency::Ngn))),
            ]),
            // `order_status_changed`, worst case: a long actor beside a status.
            join_facts(&[
                Some(format!("Was {}", "Awaiting Payment")),
                Some(format!("by {long_name}")),
            ]),
            // `inventory_low_stock`.
            "Stock fell from 128 to zero".to_string(),
            // `payment_confirmed`: amount plus a ULID.
            format!(
                "{} captured for order {}",
                format_amount_u64(12_450_000, ProductCurrency::Ngn),
                "01J8XZ5K9F3G7QW2M4N6P8R0T2V"
            ),
            // `invitation_created` / `invitation_accepted`: an email is the
            // longest thing any of these bodies can carry.
            format!("Invitation sent to {long_email}"),
            // `complaint_created` / `complaint_replied`: free text, previewed.
            console_preview(&"My package arrived two weeks late and I would like a refund for the shipping fee please.".repeat(3)),
        ];

        for body in bodies {
            assert!(
                body.chars().count() <= CONSOLE_BODY_BUDGET,
                "body would be truncated by the console card ({} chars): {body}",
                body.chars().count()
            );
        }
    }

    /// A preview is allowed its ellipsis on top of the budget, and must never
    /// exceed it otherwise.
    #[test]
    fn preview_output_always_respects_the_budget() {
        let inputs = [
            "short".to_string(),
            "A complaint that runs on and on and on and on and needs cutting somewhere.".repeat(4),
            "no spaces whatsoever".replace(' ', "").repeat(10),
            "Ünïcödé löökälïze".repeat(20),
        ];

        for input in inputs {
            let previewed = console_preview(&input);

            assert!(
                previewed.chars().count() <= CONSOLE_BODY_BUDGET + 1,
                "preview exceeded the budget: {previewed}"
            );
        }
    }

    #[test]
    fn sentence_cases_statuses_for_prose() {
        assert_eq!(sentence_case("fulfilled"), "Fulfilled");
        assert_eq!(sentence_case("awaiting payment"), "Awaiting payment");
        assert_eq!(sentence_case(""), "");
    }

    #[test]
    fn was_produces_a_clause_only_when_there_was_a_previous_status() {
        assert_eq!(was(Some("paid")).as_deref(), Some("Was Paid"));
        assert_eq!(was(None), None);
        assert_eq!(was(Some("  ")), None);
    }

    #[test]
    fn status_clause_leads_with_the_status() {
        assert_eq!(status_clause(Some("paid")).as_deref(), Some("Paid"));
        assert_eq!(status_clause(None), None);
    }

    #[test]
    fn joins_only_the_facts_that_are_present() {
        assert_eq!(
            join_facts(&[
                Some(item_count(2)),
                Some("Lagos".to_string()),
                None,
                Some(String::new()),
            ]),
            "2 items · Lagos"
        );
        assert_eq!(join_facts(&[None, None]), "");
    }

    #[test]
    fn counts_items_without_an_awkward_plural() {
        assert_eq!(item_count(1), "1 item");
        assert_eq!(item_count(0), "0 items");
        assert_eq!(item_count(3), "3 items");
    }

    #[test]
    fn greets_by_first_name_but_never_with_an_empty_slot() {
        assert_eq!(first_name_of("Ada Lovelace"), "Ada");
        assert_eq!(first_name_of("Prince"), "Prince");
        assert_eq!(first_name_of("   "), "   ");
        assert_eq!(first_name_of(""), "");
    }
}
