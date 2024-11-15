# `mrktpltsbot`

Self-hosted Marktplaats notifications for Telegram

[![Crates.io](https://img.shields.io/crates/v/mrktpltsbot?logo=rust&style=for-the-badge)](https://crates.io/crates/mrktpltsbot)
[![License](https://img.shields.io/crates/l/mrktpltsbot?style=for-the-badge)](LICENSE)
[![Build status](https://img.shields.io/github/actions/workflow/status/eigenein/mrktpltsbot/check.yaml?style=for-the-badge)](https://github.com/eigenein/mrktpltsbot/actions/workflows/check.yaml)
[![Code coverage](https://img.shields.io/codecov/c/github/eigenein/mrktpltsbot?style=for-the-badge)
](https://app.codecov.io/gh/eigenein/mrktpltsbot)

> [!CAUTION]
> This is an **unofficial bot** that **uses unofficial APIs**,
> so you take all the responsibility for any consequences of running the bot,
> for example, account or IP bans.

> [!WARNING]
> The version `2.x` is not well-tested yet.

> [!NOTE]
> I realize the documentation is not complete – this is not a deliberate choice,
> but rather the best I could do given the limited resources.
> I strive to maintain and improve it over time.

## Installation

`mrktpltsbot` is a single binary which can be installed from [crates.io](https://crates.io/crates/mrktpltsbot):

```shell
cargo install mrktpltsbot
```

## Usage

```text
Usage: mrktpltsbot [OPTIONS] --telegram-bot-token <BOT_TOKEN>

Options:
      --sentry-dsn <SENTRY_DSN>  Sentry DSN: <https://docs.sentry.io/concepts/key-terms/dsn-explainer/> [env: SENTRY_DSN]
      --db <DB>                  SQLite database path [env: DB] [default: mrktpltsbot.sqlite3]
  -h, --help                     Print help
  -V, --version                  Print version

Telegram:
      --telegram-bot-token <BOT_TOKEN>
          Telegram bot token: <https://core.telegram.org/bots/api#authorizing-your-bot> [env: TELEGRAM_BOT_TOKEN]
      --telegram-poll-timeout-secs <POLL_TIMEOUT_SECS>
          Timeout for Telegram long polling, in seconds [env: TELEGRAM_POLL_TIMEOUT_SECS] [default: 60]
      --telegram-authorize-chat-id <AUTHORIZED_CHAT_IDS>
          Authorize chat ID to use the bot [env: TELEGRAM_AUTHORIZED_CHAT_IDS]
      --telegram-heartbeat-url <telegram_heartbeat_url>
          Heartbeat URL for the Telegram bot [env: TELEGRAM_HEARTBEAT_URL]

Marktplaats:
      --marktplaats-crawl-interval-secs <CRAWL_INTERVAL_SECS>
          Crawling interval, in seconds [env: MARKTPLAATS_CRAWL_INTERVAL_SECS] [default: 60]
      --marktplaats-search-limit <SEARCH_LIMIT>
          Limit of Marktplaats search results per query [env: MARKTPLAATS_SEARCH_LIMIT] [default: 30]
      --marktplaats-heartbeat-url <marktplaats_heartbeat_url>
          Heartbeat URL for the Marktplaats crawler [env: MARKTPLAATS_HEARTBEAT_URL]
```
