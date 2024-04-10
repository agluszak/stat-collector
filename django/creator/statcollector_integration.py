from http import HTTPStatus
import json
import re
import requests
from uuid import UUID

from django.conf import settings
from django.http import HttpResponse
from django.views.decorators.http import require_POST

from .serializers import StatCollectorSerializer
from .models import StatCollector


def get_base_url_headers():
    url = settings.STAT_COLLECTOR_URL + "statistics_collector"
    headers = {"Content-Type": "application/json"}
    return url, headers


def sync_statcollector(statcollector: StatCollector):
    if statcollector.external_id is not None:
        ext_delete_statcollector(statcollector.external_id)
        statcollector.external_id = None
        statcollector.save(no_sync=True)

    response = ext_create_statcollector(statcollector)
    if response.status_code == HTTPStatus.CONFLICT:
        pattern = r"Conflict: statistics collector with name .+ and client .+ with id ([\w-]+) already exists"
        match = re.match(pattern, response.text)
        if match:
            uuid_str = match.group(1)
            ext_delete_statcollector(UUID(uuid_str))
            response = ext_create_statcollector(statcollector)

    if response.status_code == HTTPStatus.OK:
        statcollector.external_id = UUID(response.text.strip('"'))
        statcollector.save(no_sync=True)


def ext_delete_statcollector(ext_id: UUID):
    # makes no sense to handle response - server returns 200 no matter if id existed
    url, headers = get_base_url_headers()
    url += f"/{str(ext_id)}"
    requests.delete(url=url, headers=headers)


def ext_create_statcollector(statcollector: StatCollector):
    body = json.dumps(StatCollectorSerializer(statcollector).data)
    url, headers = get_base_url_headers()
    return requests.post(url=url, data=body, headers=headers)


def ext_url(statcollector: StatCollector):
    url, headers = get_base_url_headers()
    return url + f"/{statcollector.external_id}"


def ext_read_stats(statcollector: StatCollector):
    url, headers = get_base_url_headers()
    url += f"/{statcollector.external_id}/config"
    return requests.get(url=url, headers=headers)


@require_POST
def ext_email_reminder(request):
    collector_id = request.POST.get("collector_id")
    reminder_type = request.POST.get("reminder_type")
    url, headers = get_base_url_headers()
    url += f"/{collector_id}/send_emails/{reminder_type}"
    response = requests.post(url=url, headers=headers)
    return HttpResponse(status=response.status_code)
