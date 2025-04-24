FROM ubuntu:latest
LABEL authors="lukeberg"

CMD mkdir -p /opt/minecraft

COPY ./bedrock-server.zip /opt/minecraft

COPY ./target/debug/

RUN apt-get update && \
    apt-get install -y unzip && \
    unzip /opt/minecraft/bedrock-server.zip -d /opt/minecraft && \
    rm /opt/minecraft/bedrock-server.zip



ENTRYPOINT ["top", "-b"]