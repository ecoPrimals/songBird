# 🎯 Start Here - Songbird Quick Orientation

**Welcome to Songbird!** This guide gets you oriented in ~5 minutes.

## Quick Facts

- **What**: Universal service orchestrator for distributed systems
- **Language**: Rust 1.70+
- **Status**: Active development (Grade A, 90/100)
- **Crates**: 10/12 building (2 temporarily disabled)
- **Current Focus**: Documentation sprint (1,017 warnings → target <400)

## First Steps

### 1. Understand the Project (5 min)

**Read these in order**:
1. [README.md](./README.md) - Project overview & architecture
2. [STATUS.md](./STATUS.md) - Current metrics & progress
3. [ARCHITECTURE_OVERVIEW.md](./ARCHITECTURE_OVERVIEW.md) - System design

### 2. Get Set Up (10 min)

```bash
# Clone and build
git clone <repository-url>
cd songbird
cargo build

# Run tests
cargo test

# Check warnings
cargo clippy --workspace
```

### 3. Explore the Code (10 min)

**Crate Overview** (in order of importance):

| Crate | Purpose | Status |
|-------|---------|--------|
| `songbird-types` | Core types & traits | ✅ Building |
| `songbird-config` | Configuration system | ✅ Building |
| `songbird-discovery` | Service discovery | ✅ Building |
| `songbird-orchestrator` | Main orchestrator | ✅ Building |
| `songbird-observability` | Metrics & health | ✅ Building |
| `songbird-universal` | Universal protocols | ✅ Building |
| `songbird-registry` | Service registry | ✅ Building |
| `songbird-canonical` | Canonical patterns | ✅ Building |
| `songbird-network-federation` | Federation support | ✅ Building |
| `songbird-test-utils` | Testing utilities | ✅ Building |
| `songbird-cli` | Command-line interface | ❌ Disabled (corruption) |
| `songbird-primal-sdk` | SDK for primals | ❌ Disabled (corruption) |

## How to Navigate

### By Role

**New Contributor?**
1. Read [CONTRIBUTING.md](./CONTRIBUTING.md)
2. Check [STATUS.md](./STATUS.md) for current priorities
3. Look for "good first issue" tasks (see below)

**Developer?**
1. Start with [ARCHITECTURE_OVERVIEW.md](./ARCHITECTURE_OVERVIEW.md)
2. Read [QUICK_START_GUIDE.md](./QUICK_START_GUIDE.md)
3. Explore `crates/songbird-types/` for core abstractions

**User?**
1. See [QUICK_START_GUIDE.md](./QUICK_START_GUIDE.md)
2. Check `examples/` directory
3. Read API documentation (coming soon)

### By Task

**Want to help with documentation?**
- See [DOCUMENTATION_SPRINT_FINAL_SESSION_2.md](./DOCUMENTATION_SPRINT_FINAL_SESSION_2.md)
- Current priority: Adding `# Errors` sections to functions
- Target: Reduce warnings from 1,017 to <400

**Want to fix bugs?**
- Run `cargo clippy --workspace` to see warnings
- Check [STATUS.md](./STATUS.md) for known issues
- Look in individual crate READMEs

**Want to add features?**
- Review [ARCHITECTURE_OVERVIEW.md](./ARCHITECTURE_OVERVIEW.md)
- Check `specs/` directory for specifications
- Propose changes in issues first

## Current Priorities (October 2025)

### 🔥 High Priority
1. **Documentation Sprint** - Reduce warnings to <1,000 (17 away!)
2. **Fix Disabled Crates** - Re-enable songbird-cli and songbird-primal-sdk
3. **Test Coverage** - Increase from ~70% to 90%

### 🟡 Medium Priority
4. Remove production mocks
5. Migrate hardcoded endpoints to discovery
6. Add property-based tests

### 🟢 Low Priority
7. Performance optimizations
8. Additional examples
9. Enhanced monitoring

## Documentation Index

### Core Documentation
- [README.md](./README.md) - Project overview
- [STATUS.md](./STATUS.md) - Current metrics
- [ARCHITECTURE_OVERVIEW.md](./ARCHITECTURE_OVERVIEW.md) - System design
- [CONTRIBUTING.md](./CONTRIBUTING.md) - How to contribute
- [CHANGELOG.md](./CHANGELOG.md) - Version history

### Guides
- [QUICK_START_GUIDE.md](./QUICK_START_GUIDE.md) - Get started fast
- [QUICK_REFERENCE.md](./QUICK_REFERENCE.md) - Common commands
- [UNIVERSAL_AGNOSTICISM_MIGRATION_GUIDE.md](./UNIVERSAL_AGNOSTICISM_MIGRATION_GUIDE.md) - Migration guide

### Session Reports
- [DOCUMENTATION_SPRINT_FINAL_SESSION_2.md](./DOCUMENTATION_SPRINT_FINAL_SESSION_2.md) - Latest progress
- [DOCUMENTATION_SPRINT_EXTENDED_SESSION.md](./DOCUMENTATION_SPRINT_EXTENDED_SESSION.md) - Extended session
- [DOCUMENTATION_SPRINT_SESSION_1.md](./DOCUMENTATION_SPRINT_SESSION_1.md) - Initial session
- [SESSION_NOTES_OCT_14_RECOVERY.md](./SESSION_NOTES_OCT_14_RECOVERY.md) - Important lessons

### Complete Navigation
See [ROOT_DOCS_INDEX.md](./ROOT_DOCS_INDEX.md) for complete documentation index.

## Quick Commands

```bash
# Build all crates
cargo build --workspace

# Run all tests
cargo test --workspace

# Check for warnings
cargo clippy --workspace

# Format code
cargo fmt --all

# Generate docs
cargo doc --workspace --no-deps --open

# Run benchmarks
cargo bench --workspace

# Check a specific crate
cargo check -p songbird-types
cargo clippy -p songbird-orchestrator
```

## Getting Help

1. **Check documentation** - Most questions answered in docs
2. **Read session reports** - See what's been done recently
3. **Check STATUS.md** - Current priorities and known issues
4. **Review architecture** - Understanding the design helps

## Common Questions

**Q: Why are there so many warnings?**
A: We're in an active documentation sprint. Current: 1,017, Target: <400

**Q: Why are 2 crates disabled?**
A: songbird-cli and songbird-primal-sdk have corruption that needs manual fixes

**Q: Can I contribute?**
A: Yes! See [CONTRIBUTING.md](./CONTRIBUTING.md). Documentation help especially welcome!

**Q: What's the project status?**
A: Active development, Grade A (90/100). See [STATUS.md](./STATUS.md) for details.

**Q: How stable is this?**
A: 10/12 crates build and test cleanly. Core functionality is solid. Documentation is improving.

---

**Next Steps**: Read [README.md](./README.md) for full overview, then dive into [ARCHITECTURE_OVERVIEW.md](./ARCHITECTURE_OVERVIEW.md)!
