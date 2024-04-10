from django.contrib import admin
from nested_inline.admin import (
    NestedModelAdmin,
    NestedTabularInline,
)

from .models import (
    Client,
    Copy,
    Supplier,
    Placement,
    PlacementType,
    Statistic,
    StatCollector,
    Period,
)


class CopyInline(NestedTabularInline):
    model = Copy
    extra = 0
    fk_name = "placement"


class PlacementInline(NestedTabularInline):
    model = Placement
    extra = 0
    fk_name = "collector"
    inlines = (CopyInline,)


class PeriodInline(admin.TabularInline):
    model = Period
    extra = 0


@admin.register(StatCollector)
class StatCollectorAdmin(NestedModelAdmin):
    inlines = (PlacementInline, PeriodInline)
    list_display = ["name", "client", "start_date", "end_date"]
    readonly_fields = ["id", "external_id"]


# @admin.register(Copy)
# class CopyAdmin(admin.ModelAdmin):
#     pass
#
#
# @admin.register(Placement)
# class PlacementAdmin(admin.ModelAdmin):
#     pass


class DictAdmin(admin.ModelAdmin):
    list_display = ["name", "is_active"]
    list_filter = ["is_active"]

    def get_actions(self, request):
        actions = super().get_actions(request)
        if "delete_selected" in actions:
            del actions["delete_selected"]
        return actions


@admin.register(Client)
class ClientAdmin(DictAdmin):
    pass


@admin.register(Supplier)
class SupplierAdmin(DictAdmin):
    list_display = DictAdmin.list_display + ["email"]


@admin.register(PlacementType)
class PlacementTypeAdmin(DictAdmin):
    pass


@admin.register(Statistic)
class StatisticAdmin(DictAdmin):
    pass
