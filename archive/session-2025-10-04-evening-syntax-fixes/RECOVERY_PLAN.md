# 🔧 Recovery Plan - Fixing Sed Damage

**Status**: Git revert completed, starting fresh

---

## What Happened

My overly aggressive sed commands broke many files by adding `)` and `?` in wrong places:
- `use X;` became `use X)?;`
- `fn name()` became `fn name())?`
- `pub mod X;` became `pub mod X)?;`

## Solution

**Use git to revert to clean state, then apply fixes carefully.**

---

## Quick Recovery Steps

```bash
# 1. Revert songbird-core to clean state
cd /home/eastgate/Development/ecoPrimals/songbird
git checkout HEAD -- crates/songbird-core/src/

# 2. Check build status
cargo build -p songbird-core 2>&1 | tail -20

# 3. If there are still errors from earlier fixes, apply them carefully with Python script
```

---

## Proper Fix Strategy

Instead of sed, use Python with proper parsing:

```python
#!/usr/bin/env python3
import re
import glob

def fix_error_construction(filepath):
    with open(filepath, 'r') as f:
        content = f.read()
    
    original = content
    
    # Only fix SongbirdError patterns, nothing else
    # Pattern: SongbirdError::Service(Box::new(ServiceError { ... }))
    # Replace with: SongbirdError::service_error("service", "message")
    
    # Be very specific - only match the exact pattern
    content = re.sub(
        r'SongbirdError::Service\(Box::new\(\s*ServiceError\s*\{\s*service:\s*"([^"]+)"\s*\.to_string\(\),\s*message:\s*"([^"]+)"\s*\.to_string\(\),\s*[^}]*\}\s*\)\)',
        r'SongbirdError::service_error("\1", "\2")',
        content,
        flags=re.DOTALL
    )
    
    if content != original:
        with open(filepath, 'w') as f:
            f.write(content)
        return True
    return False

# Apply carefully to specific files only
```

---

## Current Status After Revert

✅ Git revert completed  
⏳ Need to check build status  
⏳ May need to re-apply some earlier fixes carefully

---

## Lesson Learned

**ALWAYS**:
1. Commit before bulk operations
2. Test on 1-2 files first
3. Use Python for complex patterns
4. Validate incrementally
5. Have rollback plan ready

**NEVER**:
1. Use sed for complex multi-pattern changes
2. Apply global replacements without testing
3. Change patterns you don't fully understand

---

## Next Steps

1. Check what state we're in after git revert
2. Identify what (if any) good changes need to be re-applied
3. Apply fixes carefully with tested Python scripts
4. Build and validate after each change

---

**Bottom Line**: Starting fresh from git is the right move. We can re-apply the good fixes carefully and avoid the sed damage.

