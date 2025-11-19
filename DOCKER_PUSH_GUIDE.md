# 📤 Push Docker Image to GitHub Container Registry

## Steps to Push Pre-Built Image

### 1. Create GitHub Personal Access Token

1. Go to: https://github.com/settings/tokens/new
2. **Note:** `Docker push to ghcr.io`
3. **Expiration:** 90 days (or longer)
4. **Scopes:** Check these boxes:
   - ✅ `write:packages` (upload packages to GitHub Package Registry)
   - ✅ `read:packages` (download packages from GitHub Package Registry)
   - ✅ `delete:packages` (optional - delete packages)
5. Click **Generate token**
6. **COPY THE TOKEN** (you won't see it again!)

### 2. Login to GitHub Container Registry

```bash
# Set your token as environment variable (safer than typing it)
export GITHUB_TOKEN="ghp_your_token_here"

# Login to ghcr.io
echo $GITHUB_TOKEN | docker login ghcr.io -u codenlighten --password-stdin
```

**Expected output:**
```
Login Succeeded
```

### 3. Push the Image

```bash
cd /mnt/storage/dev/bryan_dev/boundless-git-collab

# Push both tags
docker push ghcr.io/codenlighten/boundless-bls:latest
docker push ghcr.io/codenlighten/boundless-bls:ml-dsa
```

**Expected output:**
```
The push refers to repository [ghcr.io/codenlighten/boundless-bls]
5f70bf18a086: Pushed
d4ef6891eb15: Pushed
...
latest: digest: sha256:abc123... size: 2417
```

**Time:** 2-5 minutes (depends on upload speed)  
**Size:** ~400-600 MB compressed

### 4. Make Package Public

1. Go to: https://github.com/codenlighten?tab=packages
2. Click on **boundless-bls** package
3. Click **Package settings** (right sidebar)
4. Scroll to **Danger Zone**
5. Click **Change visibility** → **Public**
6. Confirm

**This allows Bryan to pull without authentication!**

### 5. Verify It Works

```bash
# Logout to test public access
docker logout ghcr.io

# Try pulling (should work without login)
docker pull ghcr.io/codenlighten/boundless-bls:latest
```

---

## Alternative: Use Your Personal GitHub Account

If you want to push under your own account instead of `codenlighten`:

```bash
# Build with your username
docker build -t ghcr.io/YOUR_USERNAME/boundless-bls:latest .

# Login with your token
echo $GITHUB_TOKEN | docker login ghcr.io -u YOUR_USERNAME --password-stdin

# Push
docker push ghcr.io/YOUR_USERNAME/boundless-bls:latest
```

Then update `pull-docker-image.sh` to use your URL.

---

## After Pushing

Once the image is public on ghcr.io, Bryan can simply run:

```bash
./pull-docker-image.sh
```

**Benefits:**
- ⚡ **2 minutes** total (vs 15 minutes building)
- 🔄 **Guaranteed identical** across all nodes
- 📦 **Pre-compiled** for faster deployment
- ☁️ **Free hosting** on GitHub

---

## Image Details

**Image name:** `ghcr.io/codenlighten/boundless-bls:latest`  
**Size (compressed):** ~400-600 MB  
**Size (uncompressed):** ~1.2 GB  
**Architecture:** linux/amd64  
**Contents:**
- Debian Bookworm Slim base
- boundless-node binary (with ML-DSA support)
- boundless-cli binary (with ML-DSA support)
- liboqs 0.9.0 (post-quantum crypto library)
- All runtime dependencies

**Tags:**
- `latest` - Most recent build
- `ml-dsa` - Explicit ML-DSA support tag

---

## Troubleshooting

**"unauthorized: unauthenticated"**
→ Token expired or wrong scope. Create new token with `write:packages`

**"denied: permission_denied"**
→ Wrong username or token doesn't have package permissions

**"unknown blob"**
→ Network issue during upload. Try again: `docker push ghcr.io/...`

**"layer already exists" but push fails**
→ Clear Docker cache: `docker system prune -a` then rebuild

---

## Security Notes

- ✅ GitHub tokens are safer than passwords
- ✅ Token stored in environment variable (not command history)
- ✅ Can be revoked anytime at github.com/settings/tokens
- ✅ Public packages can be pulled by anyone (no auth needed)
- ⚠️ Never commit tokens to git repos
