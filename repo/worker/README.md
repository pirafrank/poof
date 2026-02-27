# Cloudflare Worker Setup

This guide covers setting up a Cloudflare Worker backed by an R2 bucket to serve
the poof APT and YUM repositories at `poof-pkgs.fpira.com`.

It was inspired by [this article](https://blog.cloudflare.com/using-cloudflare-r2-as-an-apt-yum-repository/),
yet I did not use the exactly same scripts in an effort to make the whole process
easier and more suited for the poof workflow.

## How it works

The Worker maps every URL path directly to an R2 key. For example:

```txt
poof-pkgs.fpira.com/apt/dists/stable/InRelease
  → R2 key: apt/dists/stable/InRelease

poof-pkgs.fpira.com/yum/el9/x86_64/repodata/repomd.xml
  → R2 key: yum/el9/x86_64/repodata/repomd.xml
```

## Prerequisites

- A Cloudflare account with `fpira.com` added as a zone
- [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/install-and-update/)
installed and authenticated:

  ```sh
  npm install -g wrangler
  wrangler login
  ```

## Setup

Full procedure follows.

### Step 1 - Create the R2 bucket

```sh
wrangler r2 bucket create poof-packages
```

Confirm it exists:

```sh
wrangler r2 bucket list
```

> This bucket name must match the `R2_BUCKET_NAME` GitHub secret name
in [packages.yml](.github/workflows/packages.yml) Actions workflow.

### Step 2 - Create the Worker project

Create a directory for the Worker alongside the repository (or anywhere convenient):

```sh
mkdir poof-pkg-worker && cd poof-pkg-worker
```

- Create `wrangler.toml`
- Create the Worker script at `src/index.js`

### Step 3 - Register DNS

Register `poof-pkgs.fpira.com` as a custom domain in Cloudflare DNS.

### Step 4 - Deploy

```sh
wrangler deploy
```

Wrangler CLI will:

1. Upload the Worker script
2. Bind the `poof-packages` R2 bucket to it

Verify the deployment:

```sh
wrangler deployments list
```

### Step 5 - Get secrets for GitHub Actions

The [packages.yml](.github/workflows/packages.yml) GitHub Actions workflow
uploads to R2 using the AWS-compatible S3 API. It needs a dedicated API token.

1. Go to **Cloudflare Dashboard → R2 → Manage R2 API Tokens**
2. Click **Create API token**
3. Set permissions to **Object Read & Write** scoped to the `poof-packages`
bucket (for security reasons, we make it scoped with the least priviledges)
4. Save the **Access Key ID** and **Secret Access Key**

Set the following secrets in *Repository settings → Secrets and variables → Actions*:

| Secret | Value |
|---|---|
| `R2_ACCESS_KEY_ID` | Access Key ID from the token above |
| `R2_SECRET_ACCESS_KEY` | Secret Access Key from the token above |
| `R2_BUCKET_NAME` | `poof-packages` |
| `R2_ACCOUNT_ID` | Your Cloudflare account ID (found in the dashboard right sidebar) |

Also, we need to add a (new) dedicated GPG key to sign packages:

| Secret | Value |
|---|---|
| `GPG_PRIVATE_KEY` | Output of `--export-secret-keys` above |
| `GPG_PASSPHRASE` | Passphrase chosen during key generation |
| `GPG_KEY_ID` | The key ID (e.g. `ABCD1234EFGH5678`) |

<details>

<summary>If you do not have a have a GPG key pair, generate a new one</summary>

#### How to generate a new GPG key pair

```sh
# Generate a new key (use "poof repository" as the name, no expiry)
gpg --full-generate-key

# Find the key ID
gpg --list-secret-keys --keyid-format LONG

# Export the private key (ASCII-armored) - store this as the secret
gpg --armor --export-secret-keys THE_KEY_ID

# Export the public key - this is what goes into the bucket as gpg.pub
gpg --armor --export THE_KEY_ID

# Important! Generate now a revocation certificate
gpg --gen-revoke --armor --output revocation-cert.asc THE_KEY_ID
```

Nevertheless, store the private key and the revocation certificate securely.
Otherwise, an attacker may sign malicious packages on your behalf or revoke
your key.

</details>

<br/>

## Publish

Run the [packages.yml](.github/workflows/packages.yml) pipeline. It will take
care of the rest.
