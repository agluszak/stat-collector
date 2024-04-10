# Generated by Django 5.0.3 on 2024-03-20 21:26

import django.db.models.deletion
import django.db.models.functions.text
import uuid
from django.db import migrations, models


class Migration(migrations.Migration):

    initial = True

    dependencies = []

    operations = [
        migrations.CreateModel(
            name="Client",
            fields=[
                (
                    "id",
                    models.UUIDField(
                        default=uuid.uuid4,
                        editable=False,
                        primary_key=True,
                        serialize=False,
                    ),
                ),
                ("name", models.CharField(max_length=255, unique=True)),
                ("is_active", models.BooleanField(default=True)),
            ],
            options={
                "ordering": ["-is_active", "name"],
                "abstract": False,
            },
        ),
        migrations.CreateModel(
            name="Placement",
            fields=[
                (
                    "id",
                    models.UUIDField(
                        default=uuid.uuid4,
                        editable=False,
                        primary_key=True,
                        serialize=False,
                    ),
                ),
            ],
        ),
        migrations.CreateModel(
            name="PlacementType",
            fields=[
                (
                    "id",
                    models.UUIDField(
                        default=uuid.uuid4,
                        editable=False,
                        primary_key=True,
                        serialize=False,
                    ),
                ),
                ("name", models.CharField(max_length=255, unique=True)),
                ("is_active", models.BooleanField(default=True)),
            ],
            options={
                "ordering": ["-is_active", "name"],
                "abstract": False,
            },
        ),
        migrations.CreateModel(
            name="Statistic",
            fields=[
                (
                    "id",
                    models.UUIDField(
                        default=uuid.uuid4,
                        editable=False,
                        primary_key=True,
                        serialize=False,
                    ),
                ),
                ("name", models.CharField(max_length=255, unique=True)),
                ("is_active", models.BooleanField(default=True)),
            ],
            options={
                "ordering": ["-is_active", "name"],
                "abstract": False,
            },
        ),
        migrations.CreateModel(
            name="Supplier",
            fields=[
                (
                    "id",
                    models.UUIDField(
                        default=uuid.uuid4,
                        editable=False,
                        primary_key=True,
                        serialize=False,
                    ),
                ),
                ("name", models.CharField(max_length=255, unique=True)),
                ("is_active", models.BooleanField(default=True)),
                ("email", models.EmailField(max_length=254)),
            ],
            options={
                "ordering": ["-is_active", "name"],
                "abstract": False,
            },
        ),
        migrations.CreateModel(
            name="Copy",
            fields=[
                (
                    "id",
                    models.UUIDField(
                        default=uuid.uuid4,
                        editable=False,
                        primary_key=True,
                        serialize=False,
                    ),
                ),
                ("text", models.CharField(max_length=255)),
                (
                    "placement",
                    models.ForeignKey(
                        on_delete=django.db.models.deletion.CASCADE,
                        to="creator.placement",
                    ),
                ),
            ],
            options={
                "ordering": ["text"],
            },
        ),
        migrations.AddField(
            model_name="placement",
            name="type",
            field=models.ForeignKey(
                on_delete=django.db.models.deletion.PROTECT, to="creator.placementtype"
            ),
        ),
        migrations.CreateModel(
            name="StatCollector",
            fields=[
                (
                    "id",
                    models.UUIDField(
                        default=uuid.uuid4,
                        editable=False,
                        primary_key=True,
                        serialize=False,
                    ),
                ),
                ("name", models.CharField(max_length=255, unique=True)),
                ("start_date", models.DateField()),
                ("end_date", models.DateField()),
                (
                    "client",
                    models.ForeignKey(
                        on_delete=django.db.models.deletion.PROTECT,
                        related_name="collectors",
                        to="creator.client",
                    ),
                ),
            ],
            options={
                "ordering": ["name"],
            },
        ),
        migrations.AddField(
            model_name="placement",
            name="collector",
            field=models.ForeignKey(
                on_delete=django.db.models.deletion.CASCADE, to="creator.statcollector"
            ),
        ),
        migrations.AddField(
            model_name="placement",
            name="statistics",
            field=models.ManyToManyField(to="creator.statistic"),
        ),
        migrations.AddField(
            model_name="placement",
            name="suppliers",
            field=models.ManyToManyField(to="creator.supplier"),
        ),
        migrations.AddConstraint(
            model_name="statcollector",
            constraint=models.UniqueConstraint(
                django.db.models.functions.text.Lower("name"),
                name="statcollector_unique_name_insensitive",
            ),
        ),
    ]