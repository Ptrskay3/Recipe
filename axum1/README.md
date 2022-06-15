## Axum backend - proof of concept

Technical to-do list:

- guest session, more ergonomic handling
- keep session alive, probably make a middleware layer that resets to the default expire duration
- validate user input from forms
- refactor the giant code blob from main
- route paths should be less random
- CI + CD, better local tooling, deploy
- decision about the `Queryable` and similar reinvent-the-wheel feeling features
- confirm email on register?
- db design
- consider making a simple stateless REST API too
- OAuth2 based on https://github.com/tokio-rs/axum/tree/main/examples/oauth ?
- Static file server to allow uploading files ?
