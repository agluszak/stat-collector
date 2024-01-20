use crate::db::StatCollectorId;
use crate::errors::AppError;
use crate::{db, json, schema};
use axum::extract::{Path, State};
use axum::Json;
use std::collections::BTreeMap;

use diesel::prelude::*;
use itertools::Itertools;

/// Returns the same json as the one used to create the statistics collector
#[utoipa::path(
    get,
    path = "/statistics_collector/{collector_id}/config",
    params(
        ("collector_id" = Uuid, Path, description = "Statistics collector id")
    ),
    responses(
        (status = 200, response = json::sent::StatCollector),
        (status = 404, description = "No such id", content_type = "text/html")
    )
)]
pub async fn get_collector_config(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    Path(collector_id): Path<StatCollectorId>,
) -> Result<Json<json::sent::StatCollector>, AppError> {
    let conn = pool.get().await?;
    let map = conn
        .interact(move |conn| {
            let collector = schema::statistics_collectors::table
                .find(collector_id)
                .first::<db::StatisticsCollector>(conn)
                .map_err(|_| AppError::not_found("collector", collector_id))?;

            let periods = schema::periods::table
                .filter(schema::periods::statistics_collector_id.eq(collector_id))
                .load::<db::Period>(conn)?
                .into_iter()
                .sorted_by_key(|period| period.start)
                .map(|period| period.as_json())
                .collect_vec();

            let periods_sort_keys = periods
                .iter()
                .enumerate()
                .map(|(i, period)| (period.id, i))
                .collect::<BTreeMap<_, _>>();

            let placement_types = schema::placement_types::table
                .filter(schema::placement_types::statistics_collector_id.eq(collector_id))
                .load::<db::PlacementType>(conn)?;

            let mut json_placement_types = Vec::new();

            for placement_type in placement_types {
                let suppliers = schema::suppliers::table
                    .filter(schema::suppliers::placement_type_id.eq(placement_type.id))
                    .load::<db::Supplier>(conn)?;

                let copies = schema::copies::table
                    .filter(schema::copies::placement_type_id.eq(placement_type.id))
                    .load::<db::Copy>(conn)?
                    .into_iter()
                    .sorted_by_key(|copy| copy.id)
                    .map(|copy| copy.as_json())
                    .collect_vec();

                let stat_types = schema::statistic_types::table
                    .filter(schema::statistic_types::placement_type_id.eq(placement_type.id))
                    .load::<db::StatisticType>(conn)?
                    .into_iter()
                    .sorted_by_key(|statistic_type| statistic_type.id)
                    .map(|statistic_type| statistic_type.as_json())
                    .collect_vec();

                let mut suppliers_json = Vec::new();

                for supplier in suppliers {
                    let stats = schema::statistics::table
                        .filter(schema::statistics::supplier_id.eq(supplier.id))
                        .select(db::Statistic::as_select())
                        .load(conn)?;

                    let mut stat_types_json = Vec::new();

                    for (_, grouped_by_stat_type) in stats
                        .into_iter()
                        .sorted_by_key(|s| s.statistic_type_id)
                        .group_by(|s| s.statistic_type_id)
                        .into_iter()
                    {
                        let mut copies_json = Vec::new();

                        for (_, grouped_by_copy) in grouped_by_stat_type
                            .into_iter()
                            .sorted_by_key(|s| s.copy_id)
                            .group_by(|s| s.copy_id)
                            .into_iter()
                        {
                            let stats = grouped_by_copy
                                .into_iter()
                                .sorted_by_key(|s| periods_sort_keys[&s.period_id])
                                .map(|s| s.value)
                                .collect::<Vec<_>>();

                            copies_json.push(stats);
                        }
                        stat_types_json.push(copies_json);
                    }

                    suppliers_json.push(supplier.as_json(stat_types_json))
                }

                let placement_type = json::sent::PlacementType {
                    id: placement_type.id,
                    name: placement_type.name.clone(),
                    suppliers: suppliers_json,
                    copies,
                    statistics: stat_types,
                };

                json_placement_types.push(placement_type);
            }

            Ok::<_, AppError>(json::sent::StatCollector {
                id: collector.id,
                name: collector.name,
                client: collector.client,
                periods,
                placement_types: json_placement_types,
            })
        })
        .await??;

    Ok(Json(map))
}
