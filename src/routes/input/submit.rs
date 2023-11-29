use crate::routes::util::internal_error;
use crate::{db, schema};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::Form;
use diesel::upsert::excluded;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;
use uuid::Uuid;

// {copy_id}-{statistic_type_id}-{period_id}
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct FormKey {
    pub copy_id: i32,
    pub statistic_type_id: i32,
    pub period_id: i32,
}

impl<'de> Deserialize<'de> for FormKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts = s.split('-').collect::<Vec<&str>>();
        if parts.len() != 3 {
            return Err(serde::de::Error::custom("invalid form key"));
        }

        let copy_id = parts[0].parse::<i32>().map_err(serde::de::Error::custom)?;
        let statistic_type_id = parts[1].parse::<i32>().map_err(serde::de::Error::custom)?;
        let period_id = parts[2].parse::<i32>().map_err(serde::de::Error::custom)?;

        Ok(FormKey {
            period_id,
            copy_id,
            statistic_type_id,
        })
    }
}

// if string is empty, value is None
#[derive(Debug)]
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

/// Submit statistics for a supplier
#[utoipa::path(
    post,
    path = "/input/{uuid}",
    params(
        ("uuid" = Uuid, Path, description = "Supplier id")
    ),
    request_body(
        content = Form<BTreeMap<FormKey, FormValue>>,
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
    Path(uuid): Path<Uuid>,
    Form(form): Form<BTreeMap<FormKey, FormValue>>,
) -> Result<Redirect, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    conn.interact(move |conn| {
        // Find supplier id
        let supplier_id = schema::suppliers::table
            .filter(schema::suppliers::input_page.eq(uuid))
            .select(schema::suppliers::id)
            .first::<i32>(conn)
            .map_err(internal_error)?;

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
            .execute(conn)
            .map_err(internal_error)?;

        Ok(())
    })
    .await
    .map_err(internal_error)??;

    Ok(Redirect::to(&format!("/input/{}", uuid)))
}
