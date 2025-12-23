# 📧 Quick Guide - Commit, Clean Branches, & Release Binaries

**ecoPrimals Team Workflow Reference**

---

## ✅ COMMIT & PUSH ALL CHANGES

```bash
git add -A
git commit -m "Your message"
git push origin $(git branch --show-current)
```

---

## 🧹 CLEAN OLD BRANCHES

### Update remote refs
```bash
git fetch -p
```

### Delete local merged branches (safe)
```bash
git branch --merged | grep -v "main" | grep -v "\*" | xargs git branch -d
```

### Delete specific branch
```bash
# Local
git branch -d old-branch-name

# Remote
git push origin --delete old-branch-name
```

---

## 📦 RELEASE BINARY (Don't Push to Repo!)

### 1. Build release
```bash
cargo build --release  # or your build command
```

### 2. Create checksum
```bash
sha256sum target/release/your-binary > your-binary.sha256
```

### 3. Create & push tag
```bash
git tag -a v0.1.0-integration -m "Integration checkpoint"
git push origin v0.1.0-integration
```

### 4. Install GitHub CLI (if needed)
```bash
sudo apt install gh
gh auth login --web
```

### 5. Create release with binary
```bash
gh release create v0.1.0-integration \
  target/release/your-binary \
  your-binary.sha256 \
  --title "Integration Checkpoint" \
  --notes "Ready for testing" \
  --prerelease
```

### Share download URL with teams:
```
https://github.com/your-org/your-repo/releases/tag/v0.1.0-integration
```

---

## 🎯 WHY THIS MATTERS

✅ **Binaries in GitHub Releases (not repo)** = Clean Git history  
✅ **Tags** = Traceable checkpoints  
✅ **Clean branches** = Easier to navigate  
✅ **One-line commands** = Fast workflow  

---

## 📚 EXAMPLE

See: https://github.com/ecoPrimals/bearDog/releases/tag/v0.9.0-integration-dec23

---

## 🐻 ecoPrimals - Keep it clean!

Questions? Ask in team chat!

