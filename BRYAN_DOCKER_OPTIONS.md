# 🔑 Bryan - How to Pull the Docker Image

## Option 1: Request Public Access (Easiest for you)

Ask the admin to make the package public at:
https://github.com/codenlighten?tab=packages → boundless-bls → Settings → Change visibility to Public

Then you can simply run:
```bash
./pull-docker-image.sh
```

---

## Option 2: Use a Token (Works Now)

If the package is private, you need a GitHub Personal Access Token:

### Get a Token

1. Go to: https://github.com/settings/tokens/new
2. Note: `Pull Docker images`
3. Expiration: 90 days
4. Scopes: ✅ `read:packages`
5. Generate token
6. Copy it (looks like `ghp_xxxxx...`)

### Use the Token

```bash
# Set the token
export GITHUB_TOKEN="ghp_your_token_here"

# Run the pull script
./pull-docker-image.sh
```

The script will automatically use the token to pull the image.

---

## Option 3: Build Locally (No Token Needed)

If you don't want to deal with tokens:

```bash
./update-docker-nodes.sh
```

This builds the image from source (takes 15 minutes instead of 2).

---

## Recommended Workflow

**For now:** Use Option 2 (token) or Option 3 (build)

**Long term:** Ask admin to make package public (Option 1) so you don't need tokens

---

## Summary

| Option | Time | Requires |
|--------|------|----------|
| Pull (public) | 2 min | Package set to public |
| Pull (private) | 2 min | GitHub token |
| Build locally | 15 min | Nothing (just git pull) |

Choose whatever works best for your setup!
