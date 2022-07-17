## Axum backend - proof of concept

Technical to-do list:

- [x] guest session, more ergonomic handling
- [x] keep session alive, probably make a middleware layer that resets to the default expire duration
- [ ] validate user input from forms
- [x] refactor the giant code blob from main
- [x] route paths should be less random
- [ ] CI + CD, better local tooling, deploy
- [ ] decision about the `Queryable` and similar reinvent-the-wheel feeling features
- [x] confirm email on register?
- [x] db design
- [x] consider making a simple stateless REST API too
- [x] OAuth2 based on https://github.com/tokio-rs/axum/tree/main/examples/oauth ?
- [ ] Static file server to allow uploading files ?
- [ ] Validate `src/routes/recipe/mod.rs#InsertIngredient` quantity and quantity_unit fields
