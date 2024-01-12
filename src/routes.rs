use maud::{html, Markup};

pub mod supplier;
pub mod statistics_collector;
pub mod util;

pub async fn main_page() -> Markup {
    html! {
        h1 { "The service is working!" }
        p { "Docs are " a href="/docs" { "here" } "." }
        p { "Raw OpenAPI schema is " a href="/api.json" { "here" } "." }
    }
}
