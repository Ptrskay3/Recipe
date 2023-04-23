# Infrastructure

### Spin up minikube

```sh
minikube start --cpus=4
```

Before building the application's Docker image locally, run:

```sh
eval $(minikube -p minikube docker-env)
```

Also, you need to create a `production.yml` configuration file. Make sure to use K8s compatible network accessors, for example for Postgres host:

```
recipe-postgres-postgresql.default.svc.cluster.local
```

and use `0.0.0.0` as the application host.

### Running PostgreSQL, Redis & MeiliSearch

```sh
helm repo add bitnami https://charts.bitnami.com/bitnami
helm repo add meilisearch https://meilisearch.github.io/meilisearch-kubernetes
helm install recipe-postgres bitnami/postgresql -n default -f storage/values.yaml
helm install recipe-redis bitnami/redis -n default -f storage/values.yaml --set auth.enabled=false
helm install recipe-meilisearch meilisearch/meilisearch -n default
```
