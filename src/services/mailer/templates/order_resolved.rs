use askama::Template;

#[derive(Template)]
#[template(path = "order_resolved.html")]
pub struct OrderResolvedTemplate<'a> {
    pub name: &'a str,
    pub order_number: &'a str,
    pub dashboard_link: &'a str,
}
