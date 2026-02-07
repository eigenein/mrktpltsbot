FROM gcr.io/distroless/cc-debian12
ARG TARGETPLATFORM

LABEL org.opencontainers.image.description="Self-hosted Marktplaats notifications for Telegram"
LABEL org.opencontainers.image.authors="eigenein"
LABEL org.opencontainers.image.source="https://github.com/eigenein/mrktpltsbot"

VOLUME /data
WORKDIR /data

ENTRYPOINT ["/mrktpltsbot"]

ADD $TARGETPLATFORM/mrktpltsbot /
