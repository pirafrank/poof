# Issue #33 Evaluation: `run` Command Implementation Options

## Problem Statement Summary

Users currently cannot run a specific version of an installed binary directly through the CLI without first setting it as default via the `use` command. This creates friction when:
- Testing different versions
- Running regression tests
- Working with projects requiring specific tool versions
- Performing backward compatibility checks

## Current Architecture Understanding

Based on codebase analysis:

1. **Installation Structure:**
   - Binaries are installed in: `~/.local/share/poof/data/github.com/USER/REPO/VERSION/`
   - Symlinks are created in: `~/.local/share/poof/bin/`
   - The `use` command creates symlinks: `bin_dir/BINARY_NAME` → `data_dir/USER/REPO/VERSION/BINARY_NAME`

2. **Key Characteristics:**
   - Multiple binaries can be installed from a single repo (e.g., `yazi` has multiple executables)
   - Binary names in PATH may differ from the repo slug (e.g., `rust_exif_renamer` repo might have a binary named `renamer`)
   - Symlinks are only created when `use` is called or during installation

## Option 1: `poof run BIN_COMMAND VERSION -- args`

### Implementation Approach
1. Check if `BIN_COMMAND` exists in `bin_dir/`
2. Follow the symlink to find the currently linked version
3. Extract the repo slug from the symlink target path
4. Look in install directory for the specified `VERSION` of that repo
5. Find the binary with the same name in that version directory
6. Execute the binary with provided arguments

### Development Implications

**Pros:**
- ✅ **User-friendly**: Users work with the actual binary name they see in PATH
- ✅ **Natural workflow**: Matches how users typically interact with binaries
- ✅ **Handles multi-binary repos**: Each binary can be run separately (e.g., `poof run yazi v1.0.0 -- --help` vs `poof run yazi-fm v1.0.0 -- --help`)
- ✅ **Works with renamed binaries**: Handles cases where binary name differs from repo slug
- ✅ **Intuitive**: Users don't need to know the repo slug, just the binary name

**Cons:**
- ⚠️ **Complexity**: Requires symlink resolution, path parsing, and repo slug extraction
- ⚠️ **Dependency on symlinks**: Requires the binary to be symlinked (what if user never ran `use`?)
- ⚠️ **Edge case handling**: What if multiple repos have binaries with the same name?
- ⚠️ **Error messages**: More complex error scenarios to handle (missing symlink, wrong version, etc.)
- ⚠️ **Implementation complexity**: Need to:
  - Resolve symlink target
  - Parse path to extract `USER/REPO` slug
  - Validate version exists
  - Handle cases where symlink doesn't exist
  - Handle cases where binary name doesn't exist in target version

**Code Complexity Estimate:**
- Medium-High complexity
- Requires new utility functions for:
  - Symlink resolution
  - Path-to-slug conversion
  - Binary name matching across versions
- Error handling for multiple edge cases

### User Experience Implications

**Pros:**
- ✅ **Familiar**: Users type the binary name they know
- ✅ **Flexible**: Works with any binary name, regardless of repo structure
- ✅ **Multi-binary support**: Natural handling of repos with multiple executables
- ✅ **Discoverable**: Users can see available binaries via `poof list` or `ls ~/.local/share/poof/bin/`

**Cons:**
- ⚠️ **Requires prior setup**: Binary must be symlinked (via `use` or installation)
- ⚠️ **Ambiguity risk**: If multiple repos have same binary name, which one to use?
- ⚠️ **Less explicit**: Doesn't clearly show which repo is being executed

### Example Usage
```bash
# User-friendly: use the binary name
poof run rust_exif_renamer v1.2.0 -- --help

# Works with multi-binary repos
poof run yazi v2.0.0 -- --help
poof run yazi-fm v2.0.0 -- --help
```

---

## Option 2: `poof run SLUG VERSION -- args`

### Implementation Approach
1. Check if `SLUG` with `VERSION` exists in install directory: `data_dir/SLUG/VERSION/`
2. Find executable files in that directory
3. If single binary: execute it
4. If multiple binaries: error (cannot determine which to run)

### Development Implications

**Pros:**
- ✅ **Simple implementation**: Direct path resolution, no symlink dependency
- ✅ **No symlink requirement**: Works even if user never ran `use`
- ✅ **Explicit**: Clear which repo and version is being executed
- ✅ **Predictable**: Straightforward path construction
- ✅ **Fewer edge cases**: Less ambiguity in implementation

**Cons:**
- ❌ **Cannot handle multi-binary repos**: Explicitly stated limitation (e.g., `yazi` repo)
- ⚠️ **Less intuitive**: Requires users to know the repo slug format
- ⚠️ **Binary name discovery**: Users need to know which binary name exists in the repo
- ⚠️ **Inconsistent with PATH usage**: Users work with binary names in PATH, but must use repo slugs here

**Code Complexity Estimate:**
- Low-Medium complexity
- Simple path construction: `data_dir/SLUG/VERSION/`
- Need to handle:
  - Version validation
  - Single vs. multiple binary detection
  - Error for multi-binary repos

### User Experience Implications

**Pros:**
- ✅ **Explicit**: Clear which repo/version is being executed
- ✅ **No setup required**: Works immediately after installation
- ✅ **Consistent with other commands**: Uses repo slug format like `install`, `use`, `update`

**Cons:**
- ❌ **Major limitation**: Cannot work with repos that have multiple binaries (like `yazi`)
- ⚠️ **Less intuitive**: Users must know repo slug, not just binary name
- ⚠️ **Inconsistent mental model**: Users interact with binary names in PATH, but must use repo slugs here
- ⚠️ **Binary name discovery**: Users need to check what binaries exist in the repo
- ⚠️ **Less discoverable**: Requires knowing the exact repo slug format

### Example Usage
```bash
# Must use repo slug
poof run pirafrank/rust_exif_renamer v1.2.0 -- --help

# Fails for multi-binary repos
poof run sxyazi/yazi v2.0.0 -- --help  # ERROR: Multiple binaries found
```

---

## Comparative Analysis

### Development Effort

| Aspect | Option 1 | Option 2 |
|--------|----------|----------|
| **Implementation Complexity** | Medium-High | Low-Medium |
| **Lines of Code** | ~150-200 | ~50-80 |
| **Edge Cases** | Many (symlinks, multi-repo conflicts) | Few (multi-binary repos) |
| **Testing Complexity** | High (many scenarios) | Medium (fewer scenarios) |
| **Maintenance Burden** | Higher (complex logic) | Lower (simple logic) |

### User Experience

| Aspect | Option 1 | Option 2 |
|--------|----------|----------|
| **Intuitiveness** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Discoverability** | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Flexibility** | ⭐⭐⭐⭐⭐ | ⭐⭐ |
| **Multi-binary Support** | ✅ Yes | ❌ No |
| **Consistency with PATH** | ✅ Yes | ❌ No |
| **Setup Requirements** | Requires symlink | No setup needed |

### Feature Completeness

| Feature | Option 1 | Option 2 |
|---------|----------|----------|
| **Works with multi-binary repos** | ✅ Yes | ❌ No |
| **Works without `use` command** | ⚠️ Partial* | ✅ Yes |
| **Handles renamed binaries** | ✅ Yes | ⚠️ Limited |
| **Clear error messages** | ⚠️ Complex | ✅ Simple |

*Option 1 could be enhanced to work without symlinks by searching all repos for the binary name, but this adds complexity.

---

## Recommendation

### Primary Recommendation: **Option 1** (with enhancements)

**Rationale:**
1. **User Experience**: Significantly better UX - users work with binary names they know
2. **Feature Completeness**: Handles multi-binary repos, which is a real-world use case
3. **Future-proof**: More flexible for future enhancements
4. **Consistency**: Aligns with how users interact with binaries in their PATH

**Recommended Enhancements:**
1. **Fallback mechanism**: If symlink doesn't exist, search all installed repos for the binary name
2. **Multi-repo conflict resolution**: If multiple repos have the same binary name, provide clear error with suggestions
3. **Version validation**: Clear error if version doesn't exist for that repo
4. **Helpful error messages**: Guide users when symlink is missing or version not found

### Alternative: **Hybrid Approach**

Consider supporting both syntaxes:
- `poof run BIN_COMMAND VERSION -- args` (Option 1) - primary, user-friendly
- `poof run SLUG VERSION BINARY_NAME -- args` (Option 2 enhanced) - for explicit control

This would:
- Provide the best UX (Option 1) for most cases
- Allow explicit control for edge cases (Option 2)
- Support multi-binary repos explicitly
- Maintain backward compatibility if needed

---

## Implementation Considerations

### For Option 1:
1. **Symlink Resolution**: Use `std::fs::read_link()` to resolve symlinks
2. **Path Parsing**: Extract `USER/REPO` from path like `data_dir/github.com/USER/REPO/VERSION/BINARY`
3. **Binary Matching**: Find binary with matching name in target version directory
4. **Error Handling**: Handle missing symlinks, missing versions, ambiguous binary names
5. **Performance**: Cache symlink resolutions if needed

### For Option 2:
1. **Path Construction**: Simple `data_dir/SLUG/VERSION/` construction
2. **Binary Detection**: Use existing `find_exec_files_in_dir()` function
3. **Multi-binary Error**: Clear error message when multiple binaries found
4. **Version Validation**: Check version exists before attempting execution

---

## Conclusion

**Option 1** provides superior user experience and feature completeness, despite higher implementation complexity. The complexity is manageable and the benefits significantly outweigh the costs. The ability to handle multi-binary repos and work with familiar binary names makes it the better long-term choice.

**Option 2** is simpler to implement but has a critical limitation (no multi-binary support) and provides a less intuitive user experience. It may be suitable as a fallback or for explicit control, but should not be the primary interface.
