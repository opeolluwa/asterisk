use askama::Template;

#[derive(Template)]
#[template(path = "transaction.html")]
pub struct TransactionTemplate<'a> {
    pub name: &'a str,
    pub transaction_reference: &'a str,
    pub items: &'a [TransactionLineItem<'a>],
    pub total: &'a str,
}

pub struct TransactionLineItem<'a> {
    pub description: &'a str,
    pub quantity: &'a str,
    pub amount: &'a str,
}

/// Everything needed to render a customer's transaction email.
///
/// Owned rather than borrowed so the caller (the order service) can build it
/// from the checkout summary and hand it over without keeping the intermediate
/// strings alive; [`TransactionTemplate`] borrows from this when rendering.
#[derive(Debug, Clone)]
pub struct TransactionEmail {
    pub transaction_reference: String,
    pub total: String,
    pub items: Vec<TransactionLineItemInput>,
}

#[derive(Debug, Clone)]
pub struct TransactionLineItemInput {
    pub description: String,
    pub quantity: String,
    pub amount: String,
}
