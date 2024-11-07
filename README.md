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

## Configuration

`mrktpltsbot` supports conventional CLI options as well as environment variables.
On launch, it also automatically loads `.env` from the working directory.

See `mrktpltsbot --help` for the complete list of options,
including the corresponding environment variable names.

> [!TIP]
> Right now, the only required option is a
> [Telegram bot token](https://core.telegram.org/bots/api#authorizing-your-bot).

> [!IMPORTANT]
> `mrktpltsbot` stores all the data in an SQLite database.
> By default, it creates the database file in the working directory.
