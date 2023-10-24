# Pre-requirements
- Helm
- Kubectl
- Helm chart for [otel-operator](https://github.com/open-telemetry/opentelemetry-helm-charts/blob/main/charts/opentelemetry-operator/README.md)

# How to use

## Install otel-operator
```
helm repo add open-telemetry https://open-telemetry.github.io/opentelemetry-helm-charts
helm repo add jetstack https://charts.jetstack.io
helm repo update
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.1/cert-manager.crds.yaml
helm install cert-manager jetstack/cert-manager  --namespace cert-manager --create-namespace --version v1.13.1
helm install opentelemetry-operator open-telemetry/opentelemetry-operator
```

## Install otel-collector
```
kubectl apply -f daemonset.yaml
```

## Secrets

### Add secrets
Read up on secrets [here](https://kubernetes.io/docs/concepts/configuration/secret/)

```
kubectl create secret generic newrelic-key-secret --from-literal=new_relic_license_key=XXXX
```

### Read/Edit/Delete secrets
```
kubectl get secret newrelic-key-secret -o jsonpath={.data}
kubectl edit secret newrelic-key-secret
kubectl delete secret newrelic-key-secret

```

## Uninstall otel collector
```
kubectl delete -f otel-collector/daemonset.yaml
```
