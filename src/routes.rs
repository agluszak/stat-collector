use maud::{html, Markup};

pub mod statistics_collector;
pub mod supplier;

pub async fn main_page() -> Markup {
    html! {
        h1 { "The service is working!" }
        p { "Docs are " a href="/docs" { "here" } "." }
        p { "Raw OpenAPI schema is " a href="/api.json" { "here" } "." }
    }
}
