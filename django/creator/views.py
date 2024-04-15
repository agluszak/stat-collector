import json
from http import HTTPStatus
import urllib.parse

from django.db import transaction
from django.http import HttpResponse, HttpResponseRedirect, Http404
from django.urls import reverse_lazy
from django.shortcuts import get_object_or_404
from django.views.decorators.http import require_GET
from django.views.generic import ListView, DetailView, CreateView, UpdateView
from rest_framework.views import APIView
from rest_framework.response import Response

from .forms import StatcollectorForm, PlacementForm, CopyFormSet
from .models import StatCollector, Placement
from .serializers import StatCollectorSerializer
from .statcollector_integration import ext_read_stats
from .utils import create_xls


class StatCollectorListView(ListView):
    model = StatCollector


class StatCollectorDetailView(DetailView):
    model = StatCollector
    pk_url_kwarg = "stat_id"

    def get(self, request, *args, **kwargs):
        statcollector = self.get_object()
        if statcollector.placements.count():
            return HttpResponseRedirect(
                reverse_lazy(
                    "creator:placement_details",
                    kwargs={
                        "stat_id": statcollector.id,
                        "placement_id": statcollector.placements.first().id,
                    },
                )
            )
        else:
            return HttpResponseRedirect(
                reverse_lazy(
                    "creator:placement_create", kwargs={"stat_id": statcollector.id}
                )
            )

    def delete(self, request, stat_id):
        statcollector = self.get_object()
        statcollector.delete()
        return HttpResponse(status=204)


class StatCollectorCreateView(CreateView):
    model = StatCollector
    form_class = StatcollectorForm
    template_name = "creator/statcollector_form.html"

    def get_success_url(self):
        return reverse_lazy(
            "creator:statcollector_details", kwargs={"stat_id": self.object.id}
        )


class StatCollectorUpdateView(UpdateView):
    model = StatCollector
    form_class = StatcollectorForm
    template_name = "creator/statcollector_form.html"
    pk_url_kwarg = "stat_id"

    def get_success_url(self):
        return reverse_lazy(
            "creator:statcollector_details", kwargs={"stat_id": self.object.id}
        )


class PlacementDetailView(DetailView):
    model = Placement
    pk_url_kwarg = "placement_id"

    def get_context_data(self, **kwargs):
        context = super().get_context_data(**kwargs)
        statcollector = self.get_object().collector
        context["statcollector"] = statcollector
        return context

    def delete(self, request, stat_id, placement_id):
        placement = self.get_object()
        placement.delete()
        return HttpResponse(status=204)


class PlacementCreateView(CreateView):
    model = Placement
    form_class = PlacementForm
    template_name = "creator/placement_form.html"

    def form_valid(self, form):
        form.instance.collector_id = self.kwargs["stat_id"]
        context = self.get_context_data()
        copy_formset = context["copy_formset"]
        with transaction.atomic():
            self.object = form.save()
            if copy_formset.is_valid():
                copy_formset.instance = self.object
                copy_formset.save()

        return super().form_valid(form)

    def get_success_url(self):
        return reverse_lazy(
            "creator:placement_details",
            kwargs={
                "stat_id": self.object.collector.id,
                "placement_id": self.object.id,
            },
        )

    def get_context_data(self, **kwargs):
        context = super().get_context_data(**kwargs)
        statcollector = StatCollector.objects.get(pk=self.kwargs["stat_id"])
        context["statcollector"] = statcollector
        if self.request.POST:
            context["copy_formset"] = CopyFormSet(
                self.request.POST, instance=self.object
            )
        else:
            context["copy_formset"] = CopyFormSet(instance=self.object)
        return context


class PlacementUpdateView(UpdateView):
    model = Placement
    form_class = PlacementForm
    template_name = "creator/placement_form.html"
    pk_url_kwarg = "placement_id"

    def get_success_url(self):
        return reverse_lazy(
            "creator:placement_details",
            kwargs={
                "stat_id": self.object.collector.id,
                "placement_id": self.object.id,
            },
        )

    def get_context_data(self, **kwargs):
        context = super().get_context_data(**kwargs)
        statcollector = self.get_object().collector
        context["statcollector"] = statcollector
        if self.request.POST:
            context["copy_formset"] = CopyFormSet(
                self.request.POST, instance=self.object
            )
        else:
            context["copy_formset"] = CopyFormSet(instance=self.object)
        return context

    def form_valid(self, form):
        context = self.get_context_data()
        copy_formset = context["copy_formset"]
        with transaction.atomic():
            self.object = form.save()
            if copy_formset.is_valid():
                copy_formset.save()
        return super().form_valid(form)


class StatCollectorListAPIView(APIView):
    """
    This view is meant to provide json with statcollector and related data
    that is used in communication with stat-collector - for testing purposes
    """

    def get_object(self, stat_id):
        try:
            return StatCollector.objects.get(id=stat_id)
        except StatCollector.DoesNotExist:
            raise Http404

    def get(self, request, stat_id, format=None):
        stat_collector = self.get_object(stat_id)
        serializer = StatCollectorSerializer(stat_collector)
        return Response(serializer.data)


@require_GET
def get_statistics(request, stat_id):
    stat_collector = get_object_or_404(StatCollector, id=stat_id)
    response = ext_read_stats(stat_collector)
    if response.status_code != HTTPStatus.OK:
        return HttpResponse(status=response.status_code)

    data = json.loads(response.text)

    xls_name, xls = create_xls(data)
    encoded_xls_name = urllib.parse.quote(xls_name.encode("utf-8"))
    response = HttpResponse(
        content=xls,
        content_type="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    )
    response["Content-Disposition"] = "attachment; filename={}.xlsx".format(
        encoded_xls_name
    )
    return response
