# Local HTTPS & WebAuthn Setup with `mkcert`

## Overview

WebAuthn browser APIs (`navigator.credentials.create` and `navigator.credentials.get`) strictly require a **Secure Context** (`window.isSecureContext === true`). Modern browsers (Chrome, Edge, Firefox, Safari) only consider a context secure if it is served over:

1. **HTTPS** (`https://...`)
2. **`http://localhost`** or **`http://127.0.0.1`** (explicit local loopback exception)

When developing with custom local domain names like `http://yubi.local` over plain HTTP, browsers disable WebAuthn and throw `NotAllowedError` ("Registration cancelled by user"). Furthermore, using untrusted self-signed certificates over HTTPS causes browsers and OS authenticators (Touch ID, Windows Hello, YubiKey CTAP2) to block WebAuthn.

`mkcert` solves this by creating a **locally trusted Certificate Authority (CA)** on your development machine and registering it directly into system and browser trust stores.

---

## Prerequisites & Installation

### Fedora Linux
```bash
# Install mkcert and NSS trust store utilities
sudo dnf install -y mkcert nss-tools
```

### Ubuntu / Debian Linux
```bash
sudo apt update && sudo apt install -y mkcert libnss3-tools
```

### macOS (Homebrew)
```bash
brew install mkcert nss
```

---

## Step-by-Step Setup for `yubi.local`

### Step 1: Install Local Root CA into System & Browser Stores

Run this command once on your development machine:

```bash
mkcert -install
```

This installs the `mkcert` development root CA into:
- System CA trust store (`/etc/pki/ca-trust` / `/usr/local/share/ca-certificates`)
- Browser NSS databases (Firefox, Chrome, Chromium, Brave, Edge)
- Java keytool trust stores

### Step 2: Generate Certificate for `yubi.local`

Generate a 100% trusted certificate and private key for `yubi.local`:

```bash
mkdir -p /tmp/yubi-certs
mkcert -key-file /tmp/yubi-certs/tls.key -cert-file /tmp/yubi-certs/tls.crt yubi.local
```

### Step 3: Load TLS Certificate into Kubernetes Secrets

Load the generated certificate into Kubernetes secrets for the application namespace (`yubi`) and the Cilium operator secrets namespace (`cilium-secrets`):

```bash
# Application namespace secret (used by yubi-gateway)
kubectl create namespace yubi --dry-run=client -o yaml | kubectl apply -f -
kubectl create secret tls yubi-tls-secret \
  --key=/tmp/yubi-certs/tls.key \
  --cert=/tmp/yubi-certs/tls.crt \
  -n yubi --dry-run=client -o yaml | kubectl apply -f -

# Cilium operator secret sync namespace (used by Cilium Envoy proxy)
kubectl create namespace cilium-secrets --dry-run=client -o yaml | kubectl apply -f -
kubectl create secret tls yubi-yubi-tls-secret \
  --key=/tmp/yubi-certs/tls.key \
  --cert=/tmp/yubi-certs/tls.crt \
  -n cilium-secrets --dry-run=client -o yaml | kubectl apply -f -
```

---

## Verification

1. Restart your browser.
2. Open **[https://yubi.local](https://yubi.local)**.
3. Confirm that:
   - The padlock icon is **green / valid** (no SSL warnings).
   - WebAuthn passkey registration and authentication succeed without any `NotAllowedError` or cancellation errors.
