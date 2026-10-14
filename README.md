# asterisk

Everything you need to run an e-commerce application

## Stack

- [Rust](https://www.rust-lang.org/) + [Axum](https://github.com/tokio-rs/axum) — RESTful 
  [SeaORM](https://www.sea-orm.dev/) — PostgreSQL
- [Docker Compose](https://docker.com) — infrastructure
- [just](https://just.systems/) — task runner

## Getting Started

```sh
git clone <repo-url> && cd asterisk
just cfg
just watch
```

### Dependencies

- Rust
- Docker
- Just

### Installing

### Executing program

| Command         | Description                        |
| --------------- | ---------------------------------- |
| `just run`      | Backend with hot-reload            |
| `just db-pull`  | Migrations Entity generation       |
| `just lint`     | Lint & format                      |
| `just kill`     | Stop containers and remove volumes |

Run `just` for all available commands.
