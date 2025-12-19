# 🎓 Student Onboarding - EcoPrimals Federated ML

**Time to First Task:** 10 minutes  
**Prerequisites:** Python 3.8+, pip  
**GPU Required:** None (federation provides it)

---

## What Is This?

Train machine learning models on **distributed GPUs** from your laptop—no cloud account, no expensive hardware, no complex setup.

You write standard PyTorch/TensorFlow code. The **EcoPrimals federation** handles:
- Finding available GPUs
- Deploying your code
- Running training
- Returning results with cryptographic verification

---

## Quick Start

### 1. Install Client (30 seconds)

```bash
pip install -r client/requirements.txt
cd client
pip install -e .
```

### 2. Connect to Federation (30 seconds)

```bash
# Ask your instructor for the federation URL
export SONGBIRD_URL="ws://192.168.1.144:8080"  # Example

python -m ecoprimals_client.connect
```

You'll see:
```
🎵 Connecting to Songbird...
✅ Connected to MSU-EcoPrimals Federation
   Available nodes: 6
   Total GPUs: 6 (88GB VRAM)
```

### 3. Submit Your First Task (5 minutes)

```bash
cd projects/01-mnist-digits
python submit.py
```

Watch your task train on remote GPUs and get a cryptographic receipt when complete!

---

## What's Included

### Example Projects
- **01-mnist-digits** - Handwritten digit classification (starter)
- **02-cifar10-images** - Image classification (intermediate)
- **03-sentiment-analysis** - Text classification (NLP intro)

### Documentation
- `STUDENT_GUIDE.md` - Complete walkthrough
- `TROUBLESHOOTING.md` - Common issues and fixes
- `tutorials/` - Step-by-step lessons

### For Instructors
- `INSTRUCTOR_GUIDE.md` - How to use in your course
- `class-integration/` - Assignment templates, grading rubrics

---

## Architecture

**Version 1 (Current):** Local network access
- Students connect to on-campus Songbird node
- Tasks route to federated compute
- Results return with cryptographic receipts

**Version 2 (Coming Soon):** Internet access
- Students submit from anywhere
- BearDog-secured authentication
- Same federation, remote access

---

## Learning Objectives

Students will:
1. ✅ Run ML training without local GPUs
2. ✅ Understand distributed compute coordination
3. ✅ Learn about cryptographic verification
4. ✅ Experience infrastructure sovereignty principles

---

## Need Help?

- 📖 Read `STUDENT_GUIDE.md` for detailed walkthrough
- 🐛 Check `TROUBLESHOOTING.md` for common issues
- 💬 Ask your instructor for federation URL and support

---

**Let's democratize ML compute!** 🌿✨

