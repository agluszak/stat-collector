use axum::extract::Path;
use axum::{extract::State, http::StatusCode};
use diesel::prelude::*;
use maud::{html, Markup};
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::routes::input::submit::FormKey;
use crate::routes::util::internal_error;
use crate::{db, render_html, schema};

struct InputPageData {
    collector_name: String,
    supplier: db::Supplier,
    placement_type: db::PlacementType,
    periods: Vec<db::Period>,
    copies: Vec<db::Copy>,
    statistic_types: Vec<db::StatisticType>,
    values: BTreeMap<FormKey, i32>,
}

/// Shows the input page for a supplier
#[utoipa::path(
    get,
    path = "/input/{uuid}",
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
    Path(uuid): Path<Uuid>,
) -> Result<Markup, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    let input_page_data = conn
        .interact(move |conn| {
            let (placement_type, supplier) = schema::suppliers::table
                .filter(schema::suppliers::input_page.eq(uuid))
                .inner_join(schema::placement_types::table)
                .select((db::PlacementType::as_select(), db::Supplier::as_select()))
                .first(conn)
                .map_err(internal_error)?;

            let (collector_id, collector_name): (i32, String) = schema::placement_types::table
                .filter(schema::placement_types::id.eq(placement_type.id.unwrap()))
                .inner_join(schema::statistics_collectors::table)
                .select((
                    schema::statistics_collectors::id,
                    schema::statistics_collectors::name,
                ))
                .first(conn)
                .map_err(internal_error)?;

            let periods = schema::periods::table
                .filter(schema::periods::statistics_collector_id.eq(collector_id))
                .select(db::Period::as_select())
                .load(conn)
                .map_err(internal_error)?;

            let copies = schema::copies::table
                .filter(schema::copies::placement_type_id.eq(collector_id))
                .select(db::Copy::as_select())
                .load(conn)
                .map_err(internal_error)?;

            let statistic_types = schema::statistic_types::table
                .filter(schema::statistic_types::placement_type_id.eq(collector_id))
                .select(db::StatisticType::as_select())
                .load(conn)
                .map_err(internal_error)?;

            let values = schema::statistics::table
                .filter(schema::statistics::supplier_id.eq(supplier.id.unwrap()))
                .select(db::Statistic::as_select())
                .load(conn)
                .map_err(internal_error)?
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

            Ok(InputPageData {
                collector_name,
                supplier,
                placement_type,
                periods,
                copies,
                statistic_types,
                values,
            })
        })
        .await
        .map_err(internal_error)??;

    let title = format!(
        "{} - {} for {}",
        input_page_data.placement_type.name,
        input_page_data.supplier.name,
        input_page_data.collector_name
    );

    let ok = render_html::template(
        &title,
        html! {
            h1 { (input_page_data.placement_type.name) " - " (input_page_data.supplier.name) " for " (input_page_data.collector_name)  }

            // Table should look like this:
            // | (empty) | period 1 | period 1 | period 2 | period 2 |
            // | (empty)  | stat 1   | stat 2   | stat 1   | stat 2   |
            // | copy 1   | input    | input    | input    | input    |
            // | copy 2   | input    | input    | input    | input    |

            form method="post" action=(format!("/input/{}", uuid)) {
                table {
                    tr {
                        th { "" }
                        @for period in &input_page_data.periods {
                            th colspan=(input_page_data.statistic_types.len()) { (period.name) }
                        }
                    }
                    tr {
                        th { "" }
                        @for _period in &input_page_data.periods {
                            @for statistic_type in &input_page_data.statistic_types {
                                th { (statistic_type.name) }
                            }
                        }
                    }
                    @for copy in &input_page_data.copies {
                        tr {
                            th { (copy.name) }
                            @for period in &input_page_data.periods {
                                @for statistic_type in &input_page_data.statistic_types {
                                    @let name = format!("{}-{}-{}", copy.id.unwrap(), statistic_type.id.unwrap(), period.id.unwrap());
                                    @let value = input_page_data.values.get(&FormKey {
                                        period_id: period.id.unwrap(),
                                        statistic_type_id: statistic_type.id.unwrap(),
                                        copy_id: copy.id.unwrap(),
                                    });
                                    td {
                                        input type="number" name=(name) id=(name) value=[value];
                                    }
                                }
                            }
                        }
                    }
                }
                input type="submit" value="Submit";
            }
        },
    );

    Ok(ok)
}
