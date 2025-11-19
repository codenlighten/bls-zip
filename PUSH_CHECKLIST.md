# ✅ GitHub Container Registry Setup Checklist

## What You Need to Do

Follow these steps to push the Docker image so Bryan can pull it in 2 minutes instead of building for 15 minutes.

---

## Step 1: Create GitHub Token

1. Open: https://github.com/settings/tokens/new
2. Settings:
   - **Note:** `Docker push boundless-bls`
   - **Expiration:** 90 days
   - **Scopes:** 
     - ✅ `write:packages`
     - ✅ `read:packages`
3. Click **Generate token**
4. **COPY THE TOKEN** (save it temporarily)

---

## Step 2: Login to GitHub Container Registry

```bash
# Replace ghp_YOUR_TOKEN with your actual token
export GITHUB_TOKEN="ghp_YOUR_TOKEN_HERE"

# Login (use 'codenlighten' if pushing to that org, or your username)
echo $GITHUB_TOKEN | docker login ghcr.io -u codenlighten --password-stdin
```

**Expected:**
```
Login Succeeded
```

---

## Step 3: Push the Image

```bash
cd /mnt/storage/dev/bryan_dev/boundless-git-collab

# Push both tags (takes 2-5 minutes)
docker push ghcr.io/codenlighten/boundless-bls:latest
docker push ghcr.io/codenlighten/boundless-bls:ml-dsa
```

**You'll see:**
```
Pushing layers...
latest: digest: sha256:abc123... size: 2417
```

---

## Step 4: Make Package Public

1. Go to: https://github.com/codenlighten?tab=packages
   - (Or https://github.com/YOUR_USERNAME?tab=packages if using your account)

2. Click **boundless-bls** package

3. Click **Package settings** (right side)

4. Scroll to **Danger Zone** → **Change visibility**

5. Select **Public** → Confirm

**Why:** Allows Bryan to pull without authentication!

---

## Step 5: Test It Works

```bash
# Logout to verify public access
docker logout ghcr.io

# Try pulling (should work without login now)
docker pull ghcr.io/codenlighten/boundless-bls:latest
```

**If it pulls successfully, you're done!** ✅

---

## Step 6: Tell Bryan

Send Bryan:
```
Docker image is ready! Update your nodes with:

cd /path/to/boundless-git-collab
git pull origin main
./pull-docker-image.sh

This takes 2 minutes instead of 15!
```

---

## Troubleshooting

**"unauthorized: unauthenticated"**
→ Token needs `write:packages` scope. Create new token.

**"Cannot connect to Docker daemon"**
→ Start Docker: `sudo systemctl start docker`

**"permission denied while trying to connect"**
→ Add user to docker group: `sudo usermod -aG docker $USER` then logout/login

**Push is very slow**
→ Normal for first push (~400-600 MB). Subsequent pushes are faster (only changed layers).

---

## Current Status

✅ Image built: `ghcr.io/codenlighten/boundless-bls:latest`  
✅ Image built: `ghcr.io/codenlighten/boundless-bls:ml-dsa`  
⏸️ **Waiting:** Push to GitHub Container Registry  
⏸️ **Waiting:** Make package public  

Once complete: Bryan can pull and run in **2 minutes total**! 🚀
