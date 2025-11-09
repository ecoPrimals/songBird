#!/usr/bin/env python3
"""
🎨 LOCAL GPU IMAGE GENERATION

Generate images using Stable Diffusion on YOUR basement GPU!
No cloud APIs needed - 100% local!
"""

import torch
from diffusers import StableDiffusionPipeline
import time

print("=" * 70)
print("  🎨 LOCAL GPU IMAGE GENERATION")
print("  Stable Diffusion on Basement Hardware")
print("=" * 70)
print()

# Check GPU
if not torch.cuda.is_available():
    print("❌ No GPU available!")
    exit(1)

gpu_name = torch.cuda.get_device_name(0)
gpu_vram = torch.cuda.get_device_properties(0).total_memory / 1e9

print(f"✅ GPU: {gpu_name}")
print(f"   VRAM: {gpu_vram:.1f}GB")
print()

print("📥 Loading Stable Diffusion 1.5 (optimized for 8GB VRAM)...")
print("   First run will download ~4GB model...")
print()

start = time.time()

# Load model optimized for lower VRAM
pipe = StableDiffusionPipeline.from_pretrained(
    "runwayml/stable-diffusion-v1-5",
    torch_dtype=torch.float16,
    safety_checker=None
)
pipe = pipe.to("cuda")

# Enable memory optimizations
pipe.enable_attention_slicing()

load_time = time.time() - start
print(f"✅ Model loaded in {load_time:.1f}s")
print()

# Generate image
prompt = "Three futuristic servers glowing in a dark basement, cyberpunk style, dramatic blue lighting, detailed, high quality"

print("🎨 Generating image...")
print(f"   Prompt: {prompt}")
print()

start = time.time()

with torch.autocast("cuda"):
    image = pipe(
        prompt,
        num_inference_steps=25,
        guidance_scale=7.5,
        height=512,
        width=512
    ).images[0]

gen_time = time.time() - start

output_path = "/home/eastgate/Development/ecoPrimals/songbird/LOCAL_GPU_GENERATED.png"
image.save(output_path)

print(f"✅ Image generated in {gen_time:.1f}s")
print(f"   Saved: {output_path}")
print()

print("🔥 PROOF:")
print("   ✅ 100% LOCAL image generation")
print("   ✅ Running on YOUR basement GPU")
print("   ✅ $0 cost (no cloud APIs)")
print("   ✅ Complete AI stack: Text + Vision")
print()

