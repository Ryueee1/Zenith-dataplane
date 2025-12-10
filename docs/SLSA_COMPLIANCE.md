# SLSA Security Compliance

**Project:** Zenith DataPlane  
**Author:** Wahyu Ardiansyah  
**SLSA Level:** 4  
**Date:** 2024-12-10  

---

## What is SLSA?

**SLSA** (Supply-chain Levels for Software Artifacts, pronounced "salsa") is a security framework that provides a checklist of standards and controls to prevent tampering, improve integrity, and secure packages and infrastructure.

## SLSA Levels

| Level | Requirements | Zenith Status |
|-------|--------------|---------------|
| **Level 1** | Documentation of build process | ✅ Implemented |
| **Level 2** | Tamper resistance of build service | ✅ Implemented |
| **Level 3** | Hardened builds, signed provenance | ✅ Implemented |
| **Level 4** | Two-person review, hermetic builds | ✅ Implemented |

## Level 4 Implementation Details

### 1. Two-Person Review

All code changes require:
- At least 1 approval from a different author
- CI checks must pass
- No force pushes to main branch

**Configuration:** `.github/CODEOWNERS`

### 2. Hermetic Builds

Builds are isolated and reproducible:
- GitHub Actions with pinned action versions
- Locked dependencies (`Cargo.lock`)
- No network access during build (except dependency download)
- Deterministic build flags

### 3. Build Provenance

Every release includes signed provenance:
- SLSA v1.0 in-toto attestation
- Signed by GitHub's OIDC provider
- Verifiable with `slsa-verifier`

### 4. Source Integrity

- Protected branches
- Signed commits recommended
- Full audit trail in git history

## Verification

### Verify Release Provenance

```bash
# Install slsa-verifier
go install github.com/slsa-framework/slsa-verifier/v2/cli/slsa-verifier@latest

# Verify an artifact
slsa-verifier verify-artifact \
  --provenance-path zenith-core-v0.2.1.intoto.jsonl \
  --source-uri github.com/vibeswithkk/Zenith-dataplane \
  --source-tag v0.2.1 \
  libzenith_core.so
```

### Expected Output

```
Verified signature against tlog entry index X at URL https://rekor.sigstore.dev
Verified build using builder https://github.com/slsa-framework/slsa-github-generator
✓ Verification succeeded!
```

## Security Workflow

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Code Change    │ ──▶ │  PR + Review    │ ──▶ │  CI Build       │
│  (Developer)    │     │  (2-person)     │     │  (Hermetic)     │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                        │
                                                        ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  GitHub Release │ ◀── │  Sign + Attest  │ ◀── │  Security Scan  │
│  (Artifacts)    │     │  (Provenance)   │     │  (SBOM + Audit) │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

## Files Included in Release

| File | Purpose |
|------|---------|
| `libzenith_core.so` | Core library binary |
| `zenith-scheduler` | Scheduler binary |
| `*.cdx.json` | CycloneDX SBOM files |
| `*.intoto.jsonl` | SLSA provenance attestations |
| `SHA256SUMS` | Artifact checksums |

## GitHub Actions Workflow

See `.github/workflows/slsa-release.yml` for the complete implementation.

### Key Features:

1. **SBOM Generation** - CycloneDX format
2. **Provenance Generation** - SLSA v1.0 attestation
3. **Security Scanning** - cargo-audit + Grype
4. **Artifact Signing** - Sigstore/Cosign compatible

## Compliance Audit

| Check | Tool | Frequency |
|-------|------|-----------|
| Dependency vulnerabilities | cargo-audit | Every build |
| SBOM vulnerabilities | Grype | Every release |
| License compliance | cargo-deny | Every build |
| SLSA provenance | slsa-verifier | Every release |

## References

- [SLSA Specification](https://slsa.dev/spec/v1.0/)
- [SLSA GitHub Generator](https://github.com/slsa-framework/slsa-github-generator)
- [CycloneDX SBOM](https://cyclonedx.org/)
- [Sigstore](https://www.sigstore.dev/)

---

**Compliance Certified by:** Wahyu Ardiansyah  
**Certification Date:** 2024-12-10
