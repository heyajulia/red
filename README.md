# 🍣 Red

Red is a toy implementation of Redis in Rust.

## 🚀 Usage

To start the server:

```bash
cargo run --release
```

Then you can connect to it with the Redis CLI:

```bash
redis-cli
```

And run commands like:

```
SET foo bar
GET foo
```

## 🔡 Commands

Currently implemented commands:

- `SET`
- `GET`
- `DEL`
- `PING`

## 🏗 Architecture

The server is single-threaded and handles commands sequentially. Data is stored in memory in a `HashMap`.

## Performance

Performance is not a goal of this project, but it's still interesting to see how it compares to Redis. In my very
unscientific tests, Red is ... on par with Redis! Setting, getting and deleting 100,000 keys takes using the script that
I wrote seems to take about 16.65 seconds for both Redis and Red, though it's possible that I'm doing something wrong.

## 👩🏼‍⚖️ Legal stuff

Red is licensed under the [0-clause BSD license](LICENSE).

Redis and the cube logo are registered trademarks of Redis Ltd. Any rights therein are reserved to Redis Ltd. Any use by
Red is for referential purposes only and does not indicate any sponsorship, endorsement, or affiliation between Redis
and Red.
