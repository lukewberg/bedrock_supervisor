FROM ubuntu:latest
LABEL authors="lukeberg"

CMD mkdir -p /opt/minecraft

COPY bedrock-server.zip /opt/minecraft/

COPY ./target/x86_64-unknown-linux-gnu/debug/bedrockd /bin/bedrockd

RUN bedrockd --config
RUN apt-get update
RUN apt-get install -y libcurl4t64
RUN apt-get install -y unzip
RUN unzip /opt/minecraft/bedrock-server.zip -d /opt/minecraft
RUN rm /opt/minecraft/bedrock-server.zip

EXPOSE 10000/udp
EXPOSE 10000/tcp

EXPOSE 19132/udp
EXPOSE 19132/tcp

ENTRYPOINT ["/bin/bedrockd"]
CMD []