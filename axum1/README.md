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

Emails:

```
curl "https://api.postmarkapp.com/email" \
      -X POST \
      -H "Accept: application/json" \
      -H "Content-Type: application/json" \
      -H "X-Postmark-Server-Token: b40d6f26-c6f7-4f90-9381-b5f26579e2f9" \
      -d '{
      "From": "peter.leeh@bitgap.com",
      "To": "peter.leeh@bitgap.com",
      "Subject": "Postmark test",
      "TextBody": "Hello dear Postmark user.",
      "HtmlBody": "<html><body><strong>Hello</strong> dear Postmark user.</body></html>"
    }
```
