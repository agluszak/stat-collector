use axum::extract::Path;
use axum::{extract::State, http::StatusCode};
use diesel::prelude::*;

use maud::{html, Markup};
use std::collections::BTreeMap;

use crate::routes::util::internal_error;
use crate::{db, schema};

struct ShowCollectorData {
    collector: db::StatisticsCollector,
    types_suppliers: BTreeMap<db::PlacementType, Vec<db::Supplier>>,
}

/// Displays a page with all suppliers for a statistics collector
#[utoipa::path(
    get,
    path = "/statistics_collector/{id}",
    params(
        ("id" = i32, Path, description = "Statistics collector id")
    ),
    responses(
        (status = 200, description = "Ok", content_type = "text/html"),
        (status = 404, description = "No such id", content_type = "text/html")
    )
)]
pub async fn show_statistics_collector(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    Path(id): Path<i32>,
) -> Result<Markup, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    let data = conn
        .interact(move |conn| {
            let collector = schema::statistics_collectors::table
                .find(id)
                .first::<db::StatisticsCollector>(conn)
                .map_err(internal_error)?;

            let placement_types: Vec<(db::PlacementType, db::Supplier)> =
                schema::placement_types::table
                    .filter(schema::placement_types::statistics_collector_id.eq(id))
                    .inner_join(schema::suppliers::table)
                    .select((db::PlacementType::as_select(), db::Supplier::as_select()))
                    .load(conn)
                    .map_err(internal_error)?;

            let mut types_suppliers = BTreeMap::new();
            for (placement_type, supplier) in placement_types {
                let suppliers = types_suppliers
                    .entry(placement_type)
                    .or_insert_with(std::vec::Vec::new);
                suppliers.push(supplier);
            }

            Ok(ShowCollectorData {
                collector,
                types_suppliers,
            })
        })
        .await
        .map_err(internal_error)??;

    let ok = html! {
        h1 { (data.collector.name) }

        @for (placement_type, suppliers) in data.types_suppliers {
            h2 { (placement_type.name) }
            ul {
                @for supplier in suppliers {
                    li { a href=(format!("/input/{}", supplier.input_page)) { (supplier.name) } }
                }
            }
        }
    };

    Ok(ok)
}
