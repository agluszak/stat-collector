FROM python:3.11.0-slim

WORKDIR /django

COPY . .
RUN pip install -r requirements.txt
CMD ["python", "manage.py", "migrate"]
