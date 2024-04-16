FROM python:3.11.0-slim

WORKDIR /django

COPY . .
RUN python -m pip install --upgrade pip
RUN pip install -r requirements.txt
RUN ["chmod", "+x", "./wait-for-it.sh"]
