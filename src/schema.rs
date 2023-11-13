// @generated automatically by Diesel CLI.

diesel::table! {
    copies (id) {
        id -> Int4,
        name -> Text,
        placement_type_id -> Int4,
    }
}

diesel::table! {
    periods (id) {
        id -> Int4,
        name -> Text,
        start -> Date,
        end -> Date,
        statistics_collector_id -> Int4,
    }
}

diesel::table! {
    placement_types (id) {
        id -> Int4,
        name -> Text,
        statistics_collector_id -> Int4,
    }
}

diesel::table! {
    statistic_types (id) {
        id -> Int4,
        name -> Text,
        placement_type_id -> Int4,
    }
}

diesel::table! {
    statistics (period_id, supplier_id, statistic_type_id, copy_id) {
        period_id -> Int4,
        supplier_id -> Int4,
        statistic_type_id -> Int4,
        copy_id -> Int4,
        value -> Int4,
    }
}

diesel::table! {
    statistics_collectors (id) {
        id -> Int4,
        name -> Text,
    }
}

diesel::table! {
    suppliers (id) {
        id -> Int4,
        name -> Text,
        mail -> Text,
        input_page -> Uuid,
        placement_type_id -> Int4,
    }
}

diesel::joinable!(copies -> placement_types (placement_type_id));
diesel::joinable!(periods -> statistics_collectors (statistics_collector_id));
diesel::joinable!(placement_types -> statistics_collectors (statistics_collector_id));
diesel::joinable!(statistic_types -> placement_types (placement_type_id));
diesel::joinable!(statistics -> copies (copy_id));
diesel::joinable!(statistics -> periods (period_id));
diesel::joinable!(statistics -> statistic_types (statistic_type_id));
diesel::joinable!(statistics -> suppliers (supplier_id));
diesel::joinable!(suppliers -> placement_types (placement_type_id));

diesel::allow_tables_to_appear_in_same_query!(
    copies,
    periods,
    placement_types,
    statistic_types,
    statistics,
    statistics_collectors,
    suppliers,
);
