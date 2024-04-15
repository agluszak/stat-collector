from django.core.management.base import BaseCommand
from django.contrib.auth.models import User
import os


class Command(BaseCommand):
    help = 'Create a superuser if it does not exist'

    def handle(self, *args, **options):
        username = os.environ.get('DJANGO_ADMIN_USERNAME')
        email = os.environ.get('DJANGO_ADMIN_EMAIL')
        password = os.environ.get('DJANGO_ADMIN_PASSWORD')

        if not email or not password or not username:
            self.stdout.write(self.style.ERROR('Please provide DJANGO_ADMIN_USERNAME, DJANGO_SUPERUSER_EMAIL and DJANGO_SUPERUSER_PASSWORD environment variables.'))
            return

        if User.objects.filter(username=username).exists():
            self.stdout.write(self.style.SUCCESS('Superuser already exists.'))
            return

        User.objects.create_superuser(username, email, password)
        self.stdout.write(self.style.SUCCESS('Superuser created successfully.'))