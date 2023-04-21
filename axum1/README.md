## Axum backend - proof of concept

#### How to run

You need to have Rust and Docker installed.

First, setup the development environment

```sh
just setup
```

then install `sqlx-cli`:

```sh
cargo install sqlx-cli --version=0.6.2
```

Create a `configuration/local.yml` based on the `local.example.yml`.

Now you're ready to start with

```sh
just run
```

Technical to-do list:

- [ ] Merge some migrations (we don't have a stable scheme yet)
- [x] Decision about the `Queryable`
- [?] Static file server to allow uploading files
- [x] Migrate to 0.6.0, `State` instead of `Extension`
