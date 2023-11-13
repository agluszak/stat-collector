use maud::{html, Markup, DOCTYPE};

pub fn template(title: &str, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                meta http-equiv="X-UA-Compatible" content="ie=edge";
                title { (title) }
                link rel="stylesheet" href="https://necolas.github.io/normalize.css/8.0.1/normalize.css";
            }
            body {
                (body)
            }
        }
    }
}
