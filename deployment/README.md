# Pre-requirements
- Helm
- Kubectl
- Helm chart for [otel-operator](https://github.com/open-telemetry/opentelemetry-helm-charts/blob/main/charts/opentelemetry-operator/README.md)

# How to use

## Install otel-operator
```bash
helm repo add open-telemetry https://open-telemetry.github.io/opentelemetry-helm-charts
helm repo add jetstack https://charts.jetstack.io
helm repo update
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.1/cert-manager.crds.yaml
helm install cert-manager jetstack/cert-manager  --namespace cert-manager --create-namespace --version v1.13.1
helm install opentelemetry-operator open-telemetry/opentelemetry-operator <-n prod>
```

## Secrets

### Add secrets
Read up on secrets [here](https://kubernetes.io/docs/concepts/configuration/secret/)

```bash
kubectl create secret generic newrelic-key-secret --from-literal=new_relic_license_key=XXXX
```

### Read/Edit/Delete secrets
```bash
kubectl get secret newrelic-key-secret -o jsonpath={.data}
kubectl edit secret newrelic-key-secret
kubectl delete secret newrelic-key-secret
```

## Deploy
```bash
helm install alex-api-rs deployment --values deployment/values.yaml
helm install alex-api-rs deployment --values deployment/values.yaml -f deployment/prod-values.yaml -n prod
```

## Port forward
```bash
kubectl port-forward service/alex-api-rs 3003:3003
kubectl port-forward service/alex-api-rs 3003:3003 -n prod
```

## Upgrade
```bash
helm upgrade alex-api-rs deployment --values deployment/values.yaml
helm upgrade alex-api-rs deployment --values deployment/values.yaml -f deployment/prod-values.yaml -n prod
```

## Uninstall
```bash
helm uninstall alex-api-rs
helm uninstall alex-api-rs -n prod
```

# Maintaining chart dependencies

## Search for charts
```bash
helm search repo open-telemetry
helm search repo jetstack
```

## Update chart dependencies
```bash
helm dependency update deployment
```

## List chart dependencies
```bash
helm dependency list deployment
```