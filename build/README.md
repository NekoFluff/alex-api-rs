# Local Dev and New Relic
A `newrelic-infra.yml` file is required in this folder for collecting host metrics when running locally using docker-compose.

```
license_key: YOUR_LICENSE_KEY
```

# K8s Deployment

### Apply
```
kubectl apply -f alex-api-rs-deployment.yaml,alex-api-rs-service.yaml
```

### Delete
```
kubectl delete -f alex-api-rs-deployment.yaml,alex-api-rs-service.yaml
```

### Port Forward
```
kubectl port-forward service/alex-api-rs 3003
```
