CREATE TABLE "statistics_collectors" (
    "id" UUID PRIMARY KEY,
    "name" TEXT NOT NULL
);

CREATE TABLE "periods" (
    "id" UUID PRIMARY KEY,
    "name" TEXT NOT NULL,
    "start" DATE NOT NULL,
    "end" DATE NOT NULL,
    "statistics_collector_id" UUID NOT NULL REFERENCES "statistics_collectors"("id")
);

CREATE TABLE "placement_types" (
    "id" UUID PRIMARY KEY,
    "name" TEXT NOT NULL,
    "statistics_collector_id" UUID NOT NULL REFERENCES "statistics_collectors"("id")
);

CREATE TABLE "suppliers" (
    "id" UUID PRIMARY KEY,
    "name" TEXT NOT NULL,
    "mail" TEXT NOT NULL,
    "placement_type_id" UUID NOT NULL REFERENCES "placement_types"("id")
);

CREATE TABLE "statistic_types" (
    "id" UUID PRIMARY KEY,
    "name" TEXT NOT NULL,
    "placement_type_id" UUID NOT NULL REFERENCES "placement_types"("id")
);

CREATE TABLE "copies" (
    "id" UUID PRIMARY KEY,
    "name" TEXT NOT NULL,
    "placement_type_id" UUID NOT NULL REFERENCES "placement_types"("id")
);

CREATE TABLE "statistics" (
    "period_id" UUID NOT NULL REFERENCES "periods"("id"),
    "supplier_id" UUID NOT NULL REFERENCES "suppliers"("id"),
    "statistic_type_id" UUID NOT NULL REFERENCES "statistic_types"("id"),
    "copy_id" UUID NOT NULL REFERENCES "copies"("id"),
    "value" INTEGER NOT NULL,
    PRIMARY KEY ("period_id", "supplier_id", "statistic_type_id", "copy_id")
);
