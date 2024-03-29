use axum::extract::Path;
use axum::extract::State;
use diesel::prelude::*;
use maud::{html, Markup};
use rust_i18n::t;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::db::{StatisticsCollector, SupplierId};
use crate::routes::supplier::submit::FormKey;

use crate::errors::AppError;
use crate::logic::render_html;
use crate::logic::time::Clock;
use crate::{db, schema};

struct InputPageData {
    collector_name: String,
    client: String,
    supplier: db::Supplier,
    placement_type: db::PlacementType,
    periods: Vec<db::Period>,
    copies: Vec<db::Copy>,
    statistic_types: Vec<db::StatisticType>,
    values: BTreeMap<FormKey, i32>,
}

static DATETIME_FORMAT: &str = "%H:%M:%S %d-%m-%Y";

/// Shows the supplier page
#[utoipa::path(
    get,
    path = "/supplier/{uuid}",
    params(
        ("uuid" = Uuid, Path, description = "Supplier id")
    ),
    responses(
        (status = 200, description = "Ok", content_type = "text/html"),
        (status = 404, description = "No such id", content_type = "text/html")
    )
)]
pub async fn show_input_page(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    State(clock): State<Arc<Mutex<dyn Clock>>>,
    Path(supplier_id): Path<SupplierId>,
) -> Result<Markup, AppError> {
    let conn = pool.get().await?;
    let input_page_data = conn
        .interact(move |conn| {
            let (placement_type, supplier) = schema::suppliers::table
                .filter(schema::suppliers::id.eq(supplier_id))
                .inner_join(schema::placement_types::table)
                .select((db::PlacementType::as_select(), db::Supplier::as_select()))
                .first(conn)
                .map_err(|_| AppError::not_found("supplier", supplier_id))?;

            let collector = schema::placement_types::table
                .filter(schema::placement_types::id.eq(placement_type.id))
                .inner_join(schema::statistics_collectors::table)
                .select(schema::statistics_collectors::all_columns)
                .first::<StatisticsCollector>(conn)?;

            let collector_id = collector.id;
            let collector_name = collector.name;
            let client = collector.client;

            let periods = schema::periods::table
                .filter(schema::periods::statistics_collector_id.eq(collector_id))
                .select(db::Period::as_select())
                .load(conn)?;

            let copies = schema::copies::table
                .filter(schema::copies::placement_type_id.eq(placement_type.id))
                .select(db::Copy::as_select())
                .load(conn)?;

            let statistic_types = schema::statistic_types::table
                .filter(schema::statistic_types::placement_type_id.eq(placement_type.id))
                .select(db::StatisticType::as_select())
                .load(conn)?;

            let values = schema::statistics::table
                .filter(schema::statistics::supplier_id.eq(supplier.id))
                .select(db::Statistic::as_select())
                .load(conn)?
                .into_iter()
                .map(|statistic| {
                    (
                        FormKey {
                            period_id: statistic.period_id,
                            statistic_type_id: statistic.statistic_type_id,
                            copy_id: statistic.copy_id,
                        },
                        statistic.value,
                    )
                })
                .collect();

            Ok::<_, AppError>(InputPageData {
                client,
                collector_name,
                supplier,
                placement_type,
                periods,
                copies,
                statistic_types,
                values,
            })
        })
        .await??;

    let title = format!(
        "{} - {} / {}",
        input_page_data.placement_type.name,
        input_page_data.supplier.name,
        input_page_data.collector_name
    );

    let now = clock.lock().unwrap().now().date_naive();

    let ok = render_html::template(
        &title,
        html! {
            h1 { (input_page_data.placement_type.name) " - " (input_page_data.supplier.name) " / " (input_page_data.collector_name)  }
            h2 { (t!("client")) ":" (input_page_data.client) }

            // Table should look like this:
            // | (empty)    | copy 1 | copy 1 | copy 2 | copy 2 |
            // | (empty)    | stat 1 | stat 2 | stat 1 | stat 2 |
            // | period 1   | input  | input  | input  | input  |
            // | period 2   | input  | input  | input  | input  |

            form method="post" action=(format!("/supplier/{}", supplier_id)) {
                table {
                    tr {
                        th { "" }
                        @for copy in &input_page_data.copies {
                            th colspan=(input_page_data.statistic_types.len()) { (t!("copy")) ":" (copy.name) }
                        }
                    }
                    tr {
                        th { "" }
                        @for _copy in &input_page_data.copies {
                            @for statistic_type in &input_page_data.statistic_types {
                                th { (statistic_type.name) }
                            }
                        }
                    }
                    @for period in &input_page_data.periods {
                        tr {
                            th { (period.name) }
                            @for copy in &input_page_data.copies {
                                @for statistic_type in &input_page_data.statistic_types {
                                    @let form_key = FormKey {
                                        period_id: period.id,
                                        statistic_type_id: statistic_type.id,
                                        copy_id: copy.id,
                                    };
                                    @let name = format!("{}", form_key);
                                    @let value = input_page_data.values.get(&form_key).copied().unwrap_or(0);
                                    @let disabled = period.start > now;
                                    td {
                                        input type="number" name=(name) id=(name) value=(value) disabled[disabled];
                                    }
                                }
                            }
                        }
                    }
                }
                p {
                    (t!("last_submitted")) ": " (input_page_data.supplier.submitted_date.format(DATETIME_FORMAT))
                }
                input type="submit" value=(t!("submit"));
            }
        },
    );

    Ok(ok)
}
