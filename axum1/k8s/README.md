# Infrastructure

### Spin up minikube

```sh
minikube start --cpus=4
eval $(minikube -p minikube docker-env)
```

### PostgreSQL & Redis

```sh
helm repo add bitnami https://charts.bitnami.com/bitnami
helm install recipe-postgres bitnami/postgresql --namespace default -f storage/values.yaml
helm install recipe-redis bitnami/redis --namespace default -f storage/values.yaml --set auth.enabled=false
```

# MeiliSearch

TODO
