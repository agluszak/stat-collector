from bootstrap_datepicker_plus.widgets import DatePickerInput
from django import forms

from .models import Copy, StatCollector, Placement


class StatcollectorForm(forms.ModelForm):
    class Meta:
        model = StatCollector
        fields = "__all__"
        widgets = {
            "start_date": DatePickerInput(),
            "end_date": DatePickerInput(range_from="start_date"),
        }


class PlacementForm(forms.ModelForm):
    class Meta:
        model = Placement
        exclude = ["collector"]


CopyFormSet = forms.inlineformset_factory(
    Placement,
    Copy,
    fields=("text",),
    extra=1,
    can_delete=True,
)
