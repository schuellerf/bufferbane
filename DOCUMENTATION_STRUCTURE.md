# Documentation Structure

## 📚 Documentation Organization

All documentation is organized into three categories:
1. **Root**: User-facing guides and technical documentation
2. **docs/**: Usage guides and examples
3. **docs/planning/**: Specifications and design documents

---

## 📖 Root Documentation

### User Guides

- **[README.md](README.md)** - Project overview, quick start, feature list
- **[INSTALL.md](INSTALL.md)** - Comprehensive installation guide
- **[CHANGELOG.md](CHANGELOG.md)** - Version history and changes

### Feature Documentation

- **[CHART_ENHANCEMENTS.md](CHART_ENHANCEMENTS.md)** - Chart feature specifications
- **[INTERACTIVE_LEGEND.md](INTERACTIVE_LEGEND.md)** - Interactive chart legend guide

### Technical Documentation

- **[BUILT_IN_CLOCK_SYNC.md](BUILT_IN_CLOCK_SYNC.md)** - Clock offset compensation algorithm
- **[OFFSET_ALGORITHM_EXPLAINED.md](OFFSET_ALGORITHM_EXPLAINED.md)** - Mathematical explanation of time sync

---

## 📁 docs/

### User Documentation

- **[USAGE.md](docs/USAGE.md)** - Command-line usage and examples

---

## 📋 docs/planning/

### Specifications

- **[SPECIFICATION.md](docs/planning/SPECIFICATION.md)** - Complete technical specification
- **[SCENARIOS.md](docs/planning/SCENARIOS.md)** - Use case scenarios and requirements

### Research & Design

- **[RESEARCH.md](docs/planning/RESEARCH.md)** - Technology evaluation and decisions
- **[PHASE_SUMMARY.md](docs/planning/PHASE_SUMMARY.md)** - Implementation phases overview

### Architecture Documents

- **[SERVER_COMPONENT.md](docs/planning/SERVER_COMPONENT.md)** - Server architecture and protocol
- **[ENCRYPTION_SECURITY.md](docs/planning/ENCRYPTION_SECURITY.md)** - Security design and threat model
- **[MULTI_INTERFACE.md](docs/planning/MULTI_INTERFACE.md)** - Multi-interface monitoring design

### Technical Decisions

- **[RRD_VS_SQLITE.md](docs/planning/RRD_VS_SQLITE.md)** - Database technology comparison

---

## 🗂️ File Categories

### Keep Updated (User-Facing)

These files should be maintained and updated with new features:

```
README.md                       - Main project documentation
INSTALL.md                      - Installation instructions
CHANGELOG.md                    - Version history
docs/USAGE.md                   - Usage examples
```

### Reference (Technical)

Technical documentation for developers and contributors:

```
BUILT_IN_CLOCK_SYNC.md          - Clock sync implementation
OFFSET_ALGORITHM_EXPLAINED.md   - Algorithm deep dive
CHART_ENHANCEMENTS.md           - Chart feature specs
INTERACTIVE_LEGEND.md           - Interactive features
```

### Archive (Planning)

Historical planning and design documents (in `docs/planning/`):

```
SPECIFICATION.md                - Technical spec (reference)
RESEARCH.md                     - Technology choices
PHASE_SUMMARY.md                - Implementation roadmap
SERVER_COMPONENT.md             - Server design
ENCRYPTION_SECURITY.md          - Security model
MULTI_INTERFACE.md              - Multi-interface design
SCENARIOS.md                    - Use cases
RRD_VS_SQLITE.md                - DB decision
```

---

## 📝 Documentation Guidelines

### What Belongs Where

#### Root Level (`/`)
- End-user documentation
- Installation and setup guides
- Feature descriptions
- Technical deep-dives

**Examples**: README.md, INSTALL.md, BUILT_IN_CLOCK_SYNC.md

#### docs/
- Usage examples
- How-to guides
- Quick references

**Examples**: USAGE.md

#### docs/planning/
- Original planning documents
- Design specifications
- Architecture decisions
- Research findings

**Examples**: SPECIFICATION.md, RESEARCH.md

### Naming Conventions

- `README.md` - Project overview (root and subdirectories)
- `UPPERCASE.md` - Documentation files
- Descriptive names: `BUILT_IN_CLOCK_SYNC.md` not `clock.md`
- Avoid version/date suffixes: `INSTALL.md` not `INSTALL_v2.md`

### What NOT to Include

❌ **Intermediate/temporary docs** - Remove after completion:
- Bug fix logs (`BUGFIX_*.md`)
- Progress tracking (`PHASE2_PROGRESS.md`)
- Milestone completions (`PHASE22_COMPLETE.md`)
- Temporary workarounds (`GLIBC_COMPATIBILITY.md`)
- Build setup notes (`MUSL_BUILD_SETUP.md`)

❌ **Redundant documentation**:
- Don't duplicate information
- Reference existing docs instead
- Consolidate similar content

---

## 🔍 Finding Documentation

### By Topic

| Topic | Document |
|-------|----------|
| **Getting Started** | [README.md](README.md) |
| **Installation** | [INSTALL.md](INSTALL.md) |
| **Usage Examples** | [docs/USAGE.md](docs/USAGE.md) |
| **Chart Features** | [CHART_ENHANCEMENTS.md](CHART_ENHANCEMENTS.md) |
| **Clock Sync** | [BUILT_IN_CLOCK_SYNC.md](BUILT_IN_CLOCK_SYNC.md) |
| **Technical Spec** | [docs/planning/SPECIFICATION.md](docs/planning/SPECIFICATION.md) |
| **Security** | [docs/planning/ENCRYPTION_SECURITY.md](docs/planning/ENCRYPTION_SECURITY.md) |

### By Audience

**End Users**:
- [README.md](README.md) - Start here
- [INSTALL.md](INSTALL.md) - How to install
- [docs/USAGE.md](docs/USAGE.md) - How to use

**System Administrators**:
- [INSTALL.md](INSTALL.md) - Deployment
- `setup-server.sh` - Server setup
- `client.conf.template` - Configuration

**Developers**:
- [docs/planning/SPECIFICATION.md](docs/planning/SPECIFICATION.md) - Full spec
- [BUILT_IN_CLOCK_SYNC.md](BUILT_IN_CLOCK_SYNC.md) - Algorithm details
- [docs/planning/RESEARCH.md](docs/planning/RESEARCH.md) - Tech decisions

---

## 🧹 Maintenance

### Regular Tasks

1. **Update README.md** - When adding features
2. **Update CHANGELOG.md** - For each release
3. **Review docs/USAGE.md** - Keep examples current
4. **Archive outdated docs** - Move to `docs/planning/` if historical value

### Cleanup Triggers

Remove temporary documentation when:
- ✅ Feature is completed
- ✅ Bug is fixed
- ✅ Content is consolidated elsewhere
- ✅ Information is outdated

---

## 📊 Current Status

```
Root Documentation:     7 files (clean, organized)
docs/:                  1 file  (USAGE.md)
docs/planning/:         9 files (archived specs)

Total:                  17 documentation files
```

**Last Cleanup**: 2025-10-18
**Status**: ✅ Clean and organized

---

## 📚 Document Templates

### New Feature Documentation

```markdown
# Feature Name

## Overview
Brief description of the feature

## How to Use
Step-by-step instructions

## Examples
Code/command examples

## Technical Details
Implementation notes

## Troubleshooting
Common issues and solutions
```

### Technical Deep-Dive

```markdown
# Technical Topic

## Problem
What problem does this solve?

## Solution
How is it solved?

## Implementation
Technical details

## Benefits
Why this approach?

## References
Related documents
```

---

**Maintained**: Yes  
**Last Updated**: 2025-10-18  
**Status**: ✅ Current

