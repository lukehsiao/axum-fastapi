<h1 align="center">
    📊<br>
    FastAPI vs Axum Benchmark with Postgres
</h1>
<div align="center">
    <strong>A simple comparison of Python/FastAPI/SQLAlchemy vs Rust/Axum/sqlx.</strong>
</div>
<br>
<br>

This repo contains two implementations of a _very_ simple web server.

**Contents**
- [What the servers do](#what-the-servers-do)
- [The FastAPI server](#the-fastapi-server)
- [The Full Async FastAPI server](#the-full-async-fastapi-server)
- [The Axum server](#the-axum-server)
- [Modifying the code](#modifying-the-code)
- [Example Benchmark Results](#example-benchmark-results)
  - [FastAPI](#fastapi)
  - [FastAPI (async)](#fastapi-async)
  - [Axum](#axum)
  - [Flamegraphs](#flamegraphs)
- [What about with more uvicorn workers?](#what-about-with-more-uvicorn-workers)
- [What about coordinated omission?](#what-about-coordinated-omission)
  - [FastAPI](#fastapi-1)
  - [Axum](#axum-1)
- [Running your own](#running-your-own)
  - [Example setup](#example-setup)
- [Complaints?](#complaints)
- [License](#license)

## What the servers do

In both cases, the server fetches users from the `users` table with the following query and returns the results.

```
SELECT * FROM "users" ORDER BY user_id LIMIT 100
```

Postgres database is seeded with 2000 users using the script in `scripts/init_db.sh`.
It is run with docker, and configured to support up to 1000 connections (though both servers only use connection pools of size 5).

[SQLAlchemy](https://docs.sqlalchemy.org/en/20/core/pooling.html)

> This is why it’s perfectly fine for create_engine() to default to using a QueuePool of size five without regard to whether or not the application really needs five connections queued up - the pool would only grow to that size if the application actually used five connections concurrently, in which case the usage of a small pool is an entirely appropriate default behavior.

In Rust, we set the `max_connections` to 5 to match.

## The FastAPI server

The FastAPI server is modeled almost directly from the [FastAPI tutorial on SQL databases](https://fastapi.tiangolo.com/tutorial/sql-databases/).
When benchmarking, we run it with `uvicorn` and a single worker (the default).
While this may seem somewhat unfair (throughput and latency improve with more workers), this is [FastAPI's recommendation](https://fastapi.tiangolo.com/deployment/server-workers/) when running in docker on k8s, as many people do.

> In particular, when running on Kubernetes you will probably not want to use Gunicorn and instead run a single Uvicorn process per container...

Increasing the number of workers to `N` improves throughput and latency, but also multiplies memory usage by `N`, as each worker runs its own process.
As is typically done with FastAPI, we use SQLAlchemy and Pydantic for structured responses.

## The Full Async FastAPI server

This FastAPI server takes a different, more optimal approach of doing _everything_ asynchronously.
It deviates more from the FastAPI tutorial, but is also _very_ simple, and actually more structurally similar to the Axum server.
When benchmarking, we still run it with `uvicorn` and a single worker (the default).

## The Axum server

The Axum server is modeled almost directly from the [Axum example for sqlx and postgres](https://github.com/tokio-rs/axum/tree/503d31976f8504bba76d9ff6d3b20738eb0f3385/examples/sqlx-postgres/src).

Although Rust does have ORMs (e.g., [diesel](https://diesel.rs/), [SeaORM](https://www.sea-ql.org/SeaORM/)), the compile-time checking of SQLx means that many applications get by without a full-fledged ORM.
This repository could be modified to use diesel as well, since [Axum provides similar examples](https://github.com/tokio-rs/axum/tree/503d31976f8504bba76d9ff6d3b20738eb0f3385/examples/diesel-async-postgres).
But, that is left as an exercise to the reader.

## Modifying the code

In both cases, the code is extremely basic, and should be easy to tweak and experiment with.

## Example Benchmark Results

On my personal PC with 64 GB of DDR5 RAM and a Ryzen 7 7800X3D (8-core, 16-thread), I saw the following.
Server and postgres all running locally.

Here's a table comparing the results

| Metric                  | FastAPI | FastAPI (async) |   Axum  |
| :---------------------- | ------: | --------------: | ------: |
| Throughput (rps)        |   `612` |          `2267` | `15363` |
| 50% latency (ms)        |  `15.4` |           `2.2` |   `0.6` |
| 99% latency (ms)        |  `29.1` |           `2.5` |   `0.9` |
| 99.9% latency (ms)      |  `33.4` |           `3.1` |   `1.0` |
| Peak Memory Usage (MiB) |    `78` |            `69` |    `11` |
| Peak CPU Usage (%)      |   `7.0` |           `5.9` |  `15.9` |

Comparing to the synchronous FastAPI baseline specifically, we find the following improvements (×).

| Metric                  | FastAPI | FastAPI (async) |   Axum  |
| :---------------------- | ------: | --------------: | ------: |
| Throughput (×)          |     `1` |          `3.70` |  `25.1` |
| 50% latency (1/×)       |     `1` |           `7.0` |  `25.7` |
| 99% latency (1/×)       |     `1` |          `11.7` |  `32.3` |
| 99.9% latency (1/×)     |     `1` |          `10.8` |  `33.4` |
| Peak Memory Usage (1/×) |     `1` |           `1.1` |   `7.1` |
| Peak CPU Usage (×)      |     `1` |           `0.8` |   `2.3` |

### FastAPI

#### Details
```
oha -n 50000 -c 10 --disable-keepalive http://localhost:8000/
Summary:
  Success rate: 100.00%
  Total:        81.7200 secs
  Slowest:      0.0383 secs
  Fastest:      0.0051 secs
  Average:      0.0163 secs
  Requests/sec: 611.8453

  Total data:   490.14 MiB
  Size/request: 10
  Size/sec:     6.00 MiB

Response time histogram:
  0.005 [1]     |
  0.008 [29]    |
  0.012 [1328]  |■■
  0.015 [20848] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  0.018 [18842] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  0.022 [3972]  |■■■■■■
  0.025 [2614]  |■■■■
  0.028 [1685]  |■■
  0.032 [533]   |
  0.035 [124]   |
  0.038 [24]    |

Response time distribution:
  10.00% in 0.0130 secs
  25.00% in 0.0141 secs
  50.00% in 0.0154 secs
  75.00% in 0.0173 secs
  90.00% in 0.0217 secs
  95.00% in 0.0249 secs
  99.00% in 0.0291 secs
  99.90% in 0.0334 secs
  99.99% in 0.0374 secs

Details (average, fastest, slowest):
  DNS+dialup:   0.0001 secs, 0.0000 secs, 0.0005 secs
  DNS-lookup:   0.0000 secs, 0.0000 secs, 0.0004 secs

Status code distribution:
  [200] 50000 responses
```

### FastAPI (async)

#### Details
```
oha -n 50000 -c 10 --disable-keepalive http://localhost:8000/
Summary:
  Success rate: 100.00%
  Total:        22.0537 secs
  Slowest:      22.0526 secs
  Fastest:      0.0019 secs
  Average:      0.0044 secs
  Requests/sec: 2267.1906

  Total data:   490.14 MiB
  Size/request: 10
  Size/sec:     22.22 MiB

Response time histogram:
   0.002 [1]     |
   2.207 [49993] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
   4.412 [1]     |
   6.617 [0]     |
   8.822 [0]     |
  11.027 [0]     |
  13.232 [0]     |
  15.437 [0]     |
  17.642 [1]     |
  19.848 [1]     |
  22.053 [3]     |

Response time distribution:
  10.00% in 0.0021 secs
  25.00% in 0.0021 secs
  50.00% in 0.0022 secs
  75.00% in 0.0022 secs
  90.00% in 0.0024 secs
  95.00% in 0.0024 secs
  99.00% in 0.0025 secs
  99.90% in 0.0031 secs
  99.99% in 2.7683 secs

Details (average, fastest, slowest):
  DNS+dialup:   0.0000 secs, 0.0000 secs, 0.0005 secs
  DNS-lookup:   0.0000 secs, 0.0000 secs, 0.0004 secs

Status code distribution:
  [200] 50000 responses
```

### Axum

#### Details
```
oha -n 50000 -c 10 --disable-keepalive http://localhost:8000/
Summary:
  Success rate: 100.00%
  Total:        3.2546 secs
  Slowest:      0.0014 secs
  Fastest:      0.0003 secs
  Average:      0.0006 secs
  Requests/sec: 15362.6923

  Total data:   490.14 MiB
  Size/request: 10
  Size/sec:     150.60 MiB

Response time histogram:
  0.000 [1]     |
  0.000 [3]     |
  0.001 [813]   |■
  0.001 [24488] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  0.001 [19610] |■■■■■■■■■■■■■■■■■■■■■■■■■
  0.001 [4344]  |■■■■■
  0.001 [650]   |
  0.001 [74]    |
  0.001 [8]     |
  0.001 [4]     |
  0.001 [5]     |

Response time distribution:
  10.00% in 0.0006 secs
  25.00% in 0.0006 secs
  50.00% in 0.0006 secs
  75.00% in 0.0007 secs
  90.00% in 0.0007 secs
  95.00% in 0.0008 secs
  99.00% in 0.0009 secs
  99.90% in 0.0010 secs
  99.99% in 0.0013 secs

Details (average, fastest, slowest):
  DNS+dialup:   0.0000 secs, 0.0000 secs, 0.0004 secs
  DNS-lookup:   0.0000 secs, 0.0000 secs, 0.0003 secs

Status code distribution:
  [200] 50000 responses
```

### Flamegraphs

For the curious, there are [flamegraphs](https://www.brendangregg.com/flamegraphs.html) provided from my machine in the directories of the servers.
For rust, it was collected by running the benchmark and using [cargo-flamegraph](https://github.com/flamegraph-rs/flamegraph), while for python, it was collected using [py-spy](https://github.com/benfred/py-spy).

## What about with more uvicorn workers?

If I run

```
uvicorn app.main:app --log-level critical --host 0.0.0.0 --port 8000 --workers 16
```

both the memory usage and CPU usage increase (e.g, up to ~1200 MiB).

Then, the results look like

```
oha -n 50000 -c 10 --disable-keepalive http://localhost:8000/
Summary:
  Success rate: 100.00%
  Total:        4.7476 secs
  Slowest:      0.0030 secs
  Fastest:      0.0006 secs
  Average:      0.0009 secs
  Requests/sec: 10531.5539

  Total data:   490.14 MiB
  Size/request: 10
  Size/sec:     103.24 MiB

Response time histogram:
  0.001 [1]     |
  0.001 [11841] |■■■■■■■■■■■■■■
  0.001 [26594] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  0.001 [7971]  |■■■■■■■■■
  0.002 [2927]  |■■■
  0.002 [481]   |
  0.002 [121]   |
  0.002 [35]    |
  0.002 [12]    |
  0.003 [12]    |
  0.003 [5]     |

Response time distribution:
  10.00% in 0.0008 secs
  25.00% in 0.0008 secs
  50.00% in 0.0009 secs
  75.00% in 0.0010 secs
  90.00% in 0.0012 secs
  95.00% in 0.0013 secs
  99.00% in 0.0016 secs
  99.90% in 0.0021 secs
  99.99% in 0.0027 secs

Details (average, fastest, slowest):
  DNS+dialup:   0.0001 secs, 0.0000 secs, 0.0010 secs
  DNS-lookup:   0.0000 secs, 0.0000 secs, 0.0005 secs

Status code distribution:
  [200] 50000 responses
```

This is a significant improvement in both throughput and latency.
Not quite a linear improvement with 16× more processes, and still slower than Axum.

## What about coordinated omission?

_WARNING: Unlike the other results, this was done on a machine with only 16GB of DDR4 RAM and a AMD Ryzen 3700X._

`oha`, the load generator I'm using, does support compensating for [coordinated omission](https://redhatperf.github.io/post/coordinated-omission/).
But, if I do so, it _really_ makes FastAPI look bad.
So bad, that I'd highly suspect I'm doing something wrong, but haven't dug into it yet.

Here's what it looks like with `-q 10000` and `--latency-correction`:

| Metric           |  FastAPI |   Axum |
| :--------------- | -------: | -----: |
| Throughput (rps) |    `317` | `9920` |
| 50% latency (ms) |  `75000` | `16.2` |
| 99% latency (ms) | `151000` | `40.4` |

I think you'll agree that this looks crazy, and suggests there is something I should tweak about the setup.
If you have ideas, please reach out!

### FastAPI

```
❯ oha -n 50000 -c 10 --disable-keepalive --latency-correction -q 10000 http://localhost:8000/
Summary:
  Success rate: 100.00%
  Total:        157.5955 secs
  Slowest:      152.5937 secs
  Fastest:      0.0147 secs
  Average:      76.0228 secs
  Requests/sec: 317.2680

  Total data:   490.90 MiB
  Size/request: 10.05 KiB
  Size/sec:     3.11 MiB

Response time histogram:
    0.015 [1]    |
   15.273 [4820] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■
   30.531 [4859] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■
   45.788 [5246] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
   61.046 [5362] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
   76.304 [5037] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
   91.562 [4983] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  106.820 [5207] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  122.078 [4564] |■■■■■■■■■■■■■■■■■■■■■■■■■■■
  137.336 [5088] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  152.594 [4833] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■

Response time distribution:
  10% in 15.7830 secs
  25% in 38.8800 secs
  50% in 75.2023 secs
  75% in 113.5457 secs
  90% in 136.7149 secs
  95% in 145.2185 secs
  99% in 151.1093 secs

Details (average, fastest, slowest):
  DNS+dialup:   0.0001 secs, 0.0000 secs, 0.0011 secs
  DNS-lookup:   0.0000 secs, 0.0000 secs, 0.0003 secs

Status code distribution:
  [200] 50000 responses
```

### Axum

```
❯ oha -n 50000 -c 10 --disable-keepalive --latency-correction -q 10000 http://localhost:8000/
Summary:
  Success rate: 100.00%
  Total:        5.0403 secs
  Slowest:      0.0415 secs
  Fastest:      0.0020 secs
  Average:      0.0199 secs
  Requests/sec: 9920.0133

  Total data:   490.90 MiB
  Size/request: 10.05 KiB
  Size/sec:     97.40 MiB

Response time histogram:
  0.002 [1]     |
  0.006 [2400]  |■■■■■
  0.010 [1570]  |■■■
  0.014 [9299]  |■■■■■■■■■■■■■■■■■■■■
  0.018 [14379] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  0.022 [4971]  |■■■■■■■■■■■
  0.026 [3931]  |■■■■■■■■
  0.030 [4365]  |■■■■■■■■■
  0.034 [2941]  |■■■■■■
  0.038 [1462]  |■■■
  0.042 [4681]  |■■■■■■■■■■

Response time distribution:
  10% in 0.0105 secs
  25% in 0.0137 secs
  50% in 0.0162 secs
  75% in 0.0265 secs
  90% in 0.0371 secs
  95% in 0.0394 secs
  99% in 0.0404 secs

Details (average, fastest, slowest):
  DNS+dialup:   0.0000 secs, 0.0000 secs, 0.0011 secs
  DNS-lookup:   0.0000 secs, 0.0000 secs, 0.0004 secs

Status code distribution:
  [200] 50000 responses
```

## Running your own

I've provided a [Justfile](https://just.systems/man/en/) to help run things the way I did.
Specifically, you can set up the database with `just initdb` (you'll need docker and postgres installed).
You can run a server with `just python` or `just rust`.
You can run the benchmark with `just oha`.
Note that the number of workers, `C`, can be increased depending on how many threads your CPU has.
If you do too many, `oha` will behave oddly.
I did so using [tmux](https://github.com/tmux/tmux/wiki), but multiple shells will also work.
Monitor the system utilization of `uvicorn` or `rust-axum` however you please; I recommend [btm](https://clementtsang.github.io/bottom/0.9.6/) with the filter `cpu>0 and (uvicorn or rust-axum or docker or oha)` on the Process Widget for a nice view.

### Example setup

<div align="center">

![screenshot](assets/in-action.png)

</div>

## Complaints?

Benchmarks are hard.
If you think something is wrong or unfair, please let me know!

## License

This repository is distributed under the terms of the Blue Oak license.
Any contributions are licensed under the same license, and acknowledge via the [Developer Certificate of Origin](https://developercertificate.org/).

See [LICENSE](LICENSE) for details.
