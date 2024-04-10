from rest_framework import serializers
from .models import StatCollector, Placement, Supplier, Period


class SupplierSerializer(serializers.ModelSerializer):
    mail = serializers.EmailField(source="email")

    class Meta:
        model = Supplier
        fields = ["name", "mail"]


class PeriodSerializer(serializers.ModelSerializer):
    startDate = serializers.SerializerMethodField()
    endDate = serializers.SerializerMethodField()

    class Meta:
        model = Period
        fields = ["name", "startDate", "endDate"]

    def get_startDate(self, obj):
        return obj.start_date.strftime("%Y.%m.%d")

    def get_endDate(self, obj):
        return obj.end_date.strftime("%Y.%m.%d")


class PlacementSerializer(serializers.ModelSerializer):
    copies = serializers.StringRelatedField(many=True)
    name = serializers.SerializerMethodField()
    statistics = serializers.StringRelatedField(many=True)
    suppliers = SupplierSerializer(many=True)

    class Meta:
        model = Placement
        fields = [
            # "id",
            "name",
            "suppliers",
            "statistics",
            "copies",
        ]

    def get_name(self, obj):
        return obj.type.name


class StatCollectorSerializer(serializers.ModelSerializer):
    placementTypes = PlacementSerializer(many=True, read_only=True, source="placements")
    periods = PeriodSerializer(many=True, read_only=True)

    client = serializers.SerializerMethodField()

    class Meta:
        model = StatCollector
        fields = [
            # "id",
            "name",
            "client",
            # "start_date",
            # "end_date",
            "periodicity",
            "weekday",
            "placementTypes",
            "periods",
        ]

    def get_client(self, obj):
        return obj.client.name
