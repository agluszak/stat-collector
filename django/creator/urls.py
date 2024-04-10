from django.urls import path

from . import views
from .statcollector_integration import ext_email_reminder

app_name = "creator"

urlpatterns = [
    path("", views.StatCollectorListView.as_view(), name="statcollector_list"),
    path("new/", views.StatCollectorCreateView.as_view(), name="statcollector_create"),
    path(
        "<uuid:stat_id>/",
        views.StatCollectorDetailView.as_view(),
        name="statcollector_details",
    ),
    path(
        "<uuid:stat_id>/edit/",
        views.StatCollectorUpdateView.as_view(),
        name="statcollector_edit",
    ),
    path(
        "<uuid:stat_id>/new/",
        views.PlacementCreateView.as_view(),
        name="placement_create",
    ),
    path(
        "<uuid:stat_id>/<uuid:placement_id>/",
        views.PlacementDetailView.as_view(),
        name="placement_details",
    ),
    path(
        "<uuid:stat_id>/<uuid:placement_id>/edit/",
        views.PlacementUpdateView.as_view(),
        name="placement_edit",
    ),
    path("<uuid:stat_id>/json/", views.StatCollectorListAPIView.as_view(), name="json"),
    path("<uuid:stat_id>/xls/", views.get_statistics, name="xls"),
    path("email/", ext_email_reminder, name="send_email"),
]
