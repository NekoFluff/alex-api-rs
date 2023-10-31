# Pre-requirements
- Helm
- Kubectl
- Helm chart for [otel-operator](https://github.com/open-telemetry/opentelemetry-helm-charts/blob/main/charts/opentelemetry-operator/README.md)

# Cluster Setup

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


## AWS EKS (Fargate)

Create a cluster:
```bash
eksctl create cluster --name dev-fg --region us-east-2
eksctl create cluster --name dev-fg --region us-east-2 --fargate

```

### Install otel-operator

Update repos
```bash
helm repo add open-telemetry https://open-telemetry.github.io/opentelemetry-helm-charts
helm repo add jetstack https://charts.jetstack.io
helm repo update
```

Pull CRDs
```bash
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.1/cert-manager.crds.yaml
```

Install cert-manager
```bash
helm install cert-manager jetstack/cert-manager  --namespace cert-manager --create-namespace --version v1.13.1
```

[Fix cert-manager](https://github.com/cert-manager/cert-manager/issues/3237#issuecomment-827523656) (Fargate only).
Change the `--secure-port` and `containerPort` from 10250 to 10260
```bash
kubectl edit deployment -n cert-manager cert-manager-webhook
```

Install Issuer
```bash
kubectl apply -f issuer.yaml
```

Install otel-operator
```bash
helm install opentelemetry-operator open-telemetry/opentelemetry-operator
```


### Install Ingress Nginx

[AWS](https://kubernetes.github.io/ingress-nginx/deploy/#aws):
```bash
helm upgrade --install ingress-nginx ingress-nginx --repo https://kubernetes.github.io/ingress-nginx --namespace ingress-nginx --create-namespace
```

[Digital Ocean](https://kubernetes.github.io/ingress-nginx/deploy/#digital-ocean):
```bash
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.8.2/deploy/static/provider/do/deploy.yaml
```


## Local Cluster

### Install otel-operator

Update repos
```bash
helm repo add open-telemetry https://open-telemetry.github.io/opentelemetry-helm-charts
helm repo add jetstack https://charts.jetstack.io
helm repo update
```

Pull CRDs
```bash
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.1/cert-manager.crds.yaml
```

Install cert-manager
```bash
helm install cert-manager jetstack/cert-manager  --namespace cert-manager --create-namespace --version v1.13.1
```

Alternative cert-manager install (with CRDs)
```bash
helm install cert-manager jetstack/cert-manager  --namespace cert-manager --create-namespace --version v1.13.1 --set installCRDs=true
```

Install otel-operator

```bash
helm install opentelemetry-operator open-telemetry/opentelemetry-operator
```

### Install Ingress Nginx

```bash
helm repo add ingress-nginx https://kubernetes.github.io/ingress-nginx
helm search repo ingress-nginx
```

```bash
helm install --name nginx-ingress stable/nginx-ingress
```

# Deploy
```bash
helm install alex-api-rs deployment --values deployment/values.yaml
helm install alex-api-rs deployment --values deployment/values.yaml -f deployment/prod-values.yaml
```

## Port forward
```bash
kubectl port-forward service/alex-api-rs 3003:3003 -n dev
kubectl port-forward service/alex-api-rs 3003:3003 -n prod
```

## Testing Ingress (Local only)
```bash
curl --resolve alex-api-rs.com:80:127.0.0.1 http://alex-api-rs.com/test
```

# Upgrade
```bash
helm upgrade alex-api-rs deployment --values deployment/values.yaml -n dev
helm upgrade alex-api-rs deployment --values deployment/values.yaml -f deployment/prod-values.yaml -n prod
```

# Uninstall
```bash
helm uninstall alex-api-rs -n dev
helm uninstall alex-api-rs -n prod
```

# Maintain chart dependencies

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