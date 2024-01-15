use crate::db::{CopyId, PeriodId, StatisticTypeId, SupplierId};

use crate::errors::AppError;
use crate::{db, schema};
use axum::extract::{Path, State};

use axum::response::Redirect;
use axum::Form;
use diesel::upsert::excluded;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;
use std::fmt::Display;
use utoipa::ToSchema;
use uuid::Uuid;

// {copy_id},{statistic_type_id},{period_id}
#[derive(Debug, Ord, Clone, Copy, PartialOrd, Eq, PartialEq, ToSchema)]
pub struct FormKey {
    pub copy_id: CopyId,
    pub statistic_type_id: StatisticTypeId,
    pub period_id: PeriodId,
}

impl Display for FormKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{},{},{}",
            self.copy_id, self.statistic_type_id, self.period_id
        )
    }
}

impl<'de> Deserialize<'de> for FormKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts = s.split(',').collect::<Vec<&str>>();
        if parts.len() != 3 {
            return Err(serde::de::Error::custom("invalid form key"));
        }

        let copy_id =
            CopyId::from_uuid(parts[0].parse::<Uuid>().map_err(serde::de::Error::custom)?);
        let statistic_type_id =
            StatisticTypeId::from_uuid(parts[1].parse::<Uuid>().map_err(serde::de::Error::custom)?);
        let period_id =
            PeriodId::from_uuid(parts[2].parse::<Uuid>().map_err(serde::de::Error::custom)?);

        Ok(FormKey {
            copy_id,
            statistic_type_id,
            period_id,
        })
    }
}

// if string is empty, value is None
#[derive(Debug, ToSchema)]
pub struct FormValue(Option<i32>);

impl<'de> Deserialize<'de> for FormValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() {
            Ok(FormValue(None))
        } else {
            let i = s.parse::<i32>().map_err(serde::de::Error::custom)?;
            Ok(FormValue(Some(i)))
        }
    }
}

/// Submits statistics for a supplier. This is not meant to be used manually.
/// It's used by the supplier page.
#[utoipa::path(
    post,
    path = "/supplier/{uuid}",
    params(
        ("uuid" = Uuid, Path, description = "Supplier id")
    ),
    request_body(
        content = BTreeMap<FormKey, FormValue>,
        content_type = "application/x-www-form-urlencoded",
        description = "Statistics to submit [copy_id]-[statistic_type_id]-[period_id]=value",
        example = json!("1-1-1=234234&1-1-2=123123&1-1-3=&1-1-4=&2-1-1=34&2-1-2=&2-1-3=3&2-1-4=")
    ),
    responses(
        (status = 200, description = "Ok", content_type = "text/html"),
        (status = 404, description = "No such id", content_type = "text/html")
    )
)]
#[axum::debug_handler]
pub async fn submit_input(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    Path(supplier_id): Path<SupplierId>,
    Form(form): Form<BTreeMap<FormKey, FormValue>>,
) -> Result<Redirect, AppError> {
    let conn = pool.get().await?;
    conn.interact(move |conn| {
        let data: Vec<db::Statistic> = form
            .iter()
            .filter_map(|(k, v)| {
                v.0.map(|v| db::Statistic {
                    period_id: k.period_id,
                    supplier_id,
                    statistic_type_id: k.statistic_type_id,
                    copy_id: k.copy_id,
                    value: v,
                })
            })
            .collect();

        // Upsert statistics
        diesel::insert_into(schema::statistics::table)
            .values(&data)
            .on_conflict((
                schema::statistics::period_id,
                schema::statistics::supplier_id,
                schema::statistics::statistic_type_id,
                schema::statistics::copy_id,
            ))
            .do_update()
            .set(schema::statistics::value.eq(excluded(schema::statistics::value)))
            .execute(conn)?;

        Ok::<_, diesel::result::Error>(())
    })
    .await??;

    Ok(Redirect::to(&format!("/supplier/{}", supplier_id)))
}
