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

(Or, in a Docker container: `docker run --rm -it redis:7.2-alpine redis-cli -h host.docker.internal`.)

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

## ⚡ Performance

Performance is not a goal of this project, but it's still interesting to see how it compares to Redis.
[In my very unscientific tests](./benchmark.ts), Red seems on par with Redis. 🤯

```
% deno run --allow-net=127.0.0.1 --allow-run=lsof benchmark.ts
Benchmarking the following Redis instance:
COMMAND     PID  USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
redis-ser 64691 julia    6u  IPv4 0x2d8a37efc8875a49      0t0  TCP *:6379 (LISTEN)
redis-ser 64691 julia    7u  IPv6 0x2d8a37f4979694a9      0t0  TCP *:6379 (LISTEN)

Average RPS: 172.6
% deno run --allow-net=127.0.0.1 --allow-run=lsof benchmark.ts
Benchmarking the following Redis instance:
COMMAND   PID  USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
red     64857 julia    3u  IPv4 0x2d8a37efc8835f19      0t0  TCP localhost:6379 (LISTEN)

Average RPS: 171.2
```

## 👩🏼‍⚖️ Legal stuff

Red is licensed under the [0-clause BSD license](LICENSE).

Redis and the cube logo are registered trademarks of Redis Ltd. Any rights therein are reserved to Redis Ltd. Any use by
Red is for referential purposes only and does not indicate any sponsorship, endorsement, or affiliation between Redis
and Red.
