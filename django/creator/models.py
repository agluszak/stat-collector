from datetime import timedelta
from uuid import uuid4

from django.core.exceptions import ValidationError
from django.db import models
from django.db.models.functions import Lower
from django.utils.functional import cached_property


class Dictionary(models.Model):
    class Meta:
        abstract = True
        ordering = ["-is_active", Lower("name")]

    id = models.UUIDField(primary_key=True, default=uuid4, editable=False)
    name = models.CharField(max_length=255, unique=True)
    is_active = models.BooleanField(default=True)

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        constraint_name = f"{self._meta.model_name}_unique_name_insensitive"
        self._meta.constraints = [
            models.UniqueConstraint(Lower("name"), name=constraint_name)
        ]

    def __str__(self):
        return self.name


class Supplier(Dictionary):
    email = models.EmailField()

    def delete(self, *args, **kwargs):
        if self.placement_set.exists():
            raise ValidationError(
                "Cannot delete Supplier used in Placements.",
            )
        super().delete(*args, **kwargs)


class Statistic(Dictionary):
    def delete(self, *args, **kwargs):
        print(list(self.placement_set.all()))
        if self.placement_set.exists():
            raise models.ProtectedError(
                "Cannot delete Statistics used in Placements.",
                self.placement_set.all(),
            )
        super().delete(*args, **kwargs)


class PlacementType(Dictionary):
    pass


class Client(Dictionary):
    pass


class Periodicity(models.TextChoices):
    DAILY = "daily", "Daily"
    WEEKLY = "weekly", "Weekly"
    BIWEEKLY = "biweekly", "Biweekly"
    MONTHLY = "monthly", "Monthly"


class Weekday(models.TextChoices):
    MONDAY = "monday", "Monday"
    TUESDAY = "tuesday", "Tuesday"
    WEDNESDAY = "wednesday", "Wednesday"
    THURSDAY = "thursday", "Thursday"
    FRIDAY = "friday", "Friday"
    SATURDAY = "saturday", "Saturday"
    SUNDAY = "sunday", "Sunday"


class StatCollector(models.Model):
    class Meta:
        constraints = [
            models.UniqueConstraint(
                Lower("name"),
                name="statcollector_unique_name_insensitive",
                violation_error_message="Statistics Collector's name must be unique (case-insensitive)",
            )
        ]
        ordering = [Lower("name")]

    id = models.UUIDField(primary_key=True, default=uuid4, editable=False)
    external_id = models.UUIDField(null=True, editable=False)
    name = models.CharField(max_length=255, unique=True)
    start_date = models.DateField()
    end_date = models.DateField()
    periodicity = models.CharField(max_length=10, choices=Periodicity.choices)
    weekday = models.CharField(max_length=10, choices=Weekday.choices, blank=True)
    client = models.ForeignKey(
        Client,
        on_delete=models.PROTECT,
        related_name="collectors",
        limit_choices_to={"is_active": True},
    )

    @cached_property
    def ext_url(self):
        from .statcollector_integration import ext_url as ext_url_f

        return ext_url_f(self)

    def clean(self):
        if self.periodicity in [Periodicity.WEEKLY, Periodicity.BIWEEKLY]:
            if not self.weekday:
                raise ValidationError(
                    "Weekday field is required for weekly or biweekly periodicity."
                )
        else:
            self.weekday = ""

    def save(self, *args, **kwargs):
        from .statcollector_integration import sync_statcollector

        no_sync = kwargs.pop("no_sync", False)
        super().save(*args, **kwargs)
        if not no_sync:
            Period.create_for_collector(self)
            sync_statcollector(self)

    def delete(self, *args, **kwargs):
        from .statcollector_integration import ext_delete_statcollector

        ext_delete_statcollector(self.external_id)
        super().delete(*args, **kwargs)


class Placement(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid4, editable=False)
    collector = models.ForeignKey(
        StatCollector, on_delete=models.CASCADE, related_name="placements"
    )
    type = models.ForeignKey(
        PlacementType, on_delete=models.PROTECT, limit_choices_to={"is_active": True}
    )
    suppliers = models.ManyToManyField(Supplier, limit_choices_to={"is_active": True})
    statistics = models.ManyToManyField(Statistic, limit_choices_to={"is_active": True})

    def save(self, *args, **kwargs):
        from .statcollector_integration import sync_statcollector

        super().save(*args, **kwargs)
        sync_statcollector(self.collector)

    def delete(self, *args, **kwargs):
        from .statcollector_integration import sync_statcollector

        super().delete(*args, **kwargs)
        sync_statcollector(self.collector)


class Copy(models.Model):
    class Meta:
        ordering = ["text"]
        verbose_name_plural = "copies"

    id = models.UUIDField(primary_key=True, default=uuid4, editable=False)
    text = models.CharField(max_length=255)
    placement = models.ForeignKey(
        Placement, on_delete=models.CASCADE, related_name="copies"
    )

    def __str__(self):
        return self.text


class Period(models.Model):
    class Meta:
        ordering = ["name"]

    id = models.UUIDField(primary_key=True, default=uuid4, editable=False)
    name = models.CharField(max_length=255)
    start_date = models.DateField()
    end_date = models.DateField()
    collector = models.ForeignKey(
        StatCollector, on_delete=models.CASCADE, related_name="periods"
    )

    @classmethod
    def create_for_collector(cls, collector: StatCollector):
        dates = None
        Period.objects.filter(collector=collector).delete()
        if collector.periodicity == Periodicity.DAILY:
            dates = cls.generate_dates(
                start_date=collector.start_date, end_date=collector.end_date, interval=1
            )
        if collector.periodicity in (Periodicity.WEEKLY, Periodicity.BIWEEKLY):
            selected_weekday = [
                idx
                for idx, (value, label) in enumerate(Weekday.choices)
                if value == collector.weekday
            ][0]
            start_date = collector.start_date - timedelta(
                days=(7 - selected_weekday + collector.start_date.weekday()) % 7
            )

            if collector.periodicity == Periodicity.WEEKLY:
                dates = cls.generate_dates(
                    start_date=start_date,
                    end_date=collector.end_date,
                    interval=7,
                )
            if collector.periodicity == Periodicity.BIWEEKLY:
                dates = cls.generate_dates(
                    start_date=start_date,
                    end_date=collector.end_date,
                    interval=14,
                )
        if collector.periodicity == Periodicity.MONTHLY:
            dates = cls.generate_months(
                start_date=collector.start_date, end_date=collector.end_date
            )
        if dates is not None:
            periods = [
                cls.objects.create(
                    name=name,
                    start_date=start_date,
                    end_date=end_date,
                    collector=collector,
                )
                for name, start_date, end_date in dates
            ]
            return periods
        return dates

    @classmethod
    def generate_dates(cls, start_date, end_date, interval):
        current_date = start_date
        while current_date <= end_date:
            c_end_date = current_date + timedelta(days=interval - 1)
            if interval == 1:
                name = current_date.strftime("%Y.%m.%d")
            else:
                name = f'{current_date.strftime("%Y.%m.%d")} - {c_end_date.strftime("%m.%d")}'
            yield name, current_date, c_end_date
            current_date += timedelta(days=interval)

    @classmethod
    def generate_months(cls, start_date, end_date):
        current_date = start_date.replace(day=1)
        while current_date <= end_date:
            c_end_date = current_date.replace(month=current_date.month + 1) - timedelta(
                days=1
            )
            yield f'{current_date.strftime("%Y.%m.%d")} - {c_end_date.strftime("%m.%d")}', current_date, c_end_date
            current_date = current_date.replace(month=current_date.month + 1)

    @classmethod
    def create_for_collector_daily(cls, collector: StatCollector):
        periods = [
            cls.objects.create(
                name=date.strftime("%Y-%m-%d"),
                start_date=date,
                end_date=date,
                collector=collector,
            )
            for date in cls.generate_dates(
                start_date=collector.start_date, end_date=collector.end_date, interval=1
            )
        ]
        return periods

    @classmethod
    def create_for_collector_weekly(cls, collector: StatCollector):
        pass

    @classmethod
    def create_for_collector_biweekly(cls, collector: StatCollector):
        pass

    @classmethod
    def create_for_collector_monthly(cls, collector: StatCollector):
        pass
