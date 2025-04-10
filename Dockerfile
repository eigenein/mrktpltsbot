FROM scratch

LABEL org.opencontainers.image.description="Self-hosted Marktplaats notifications for Telegram"
LABEL org.opencontainers.image.authors="eigenein"
LABEL org.opencontainers.image.source="https://github.com/eigenein/mrktpltsbot"

ENTRYPOINT ["/mrktpltsbot"]

COPY mrktpltsbot /
