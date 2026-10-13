use askama::Template;

#[derive(Template)]
#[template(path = "payment_confirmed.html")]
pub struct PaymentConfirmedTemplate<'a> {
    pub name: &'a str,
    pub amount: &'a str,
    pub reference: &'a str,
    pub dashboard_link: &'a str,
}
