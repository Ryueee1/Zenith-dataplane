# Software Bill of Materials (SBOM) Policy

**Project:** Zenith DataPlane  
**Author:** Wahyu Ardiansyah  
**Version:** 1.0  
**Date:** 2024-12-10  

---

## Overview

This document describes the SBOM policy for the Zenith project, ensuring supply chain transparency and security compliance.

## SBOM Format

We use **CycloneDX v1.5** format for all SBOMs:
- JSON format for machine readability
- XML format available on request
- Compatible with OWASP Dependency-Track

## Generated SBOMs

| Component          | SBOM File                          | 
|--------------------|------------------------------------|
| zenith-core        | `sbom/zenith-core.cdx.json`        |
| zenith-runtime-cpu | `sbom/zenith-runtime-cpu.cdx.json` |
| zenith-runtime-gpu | `sbom/zenith-runtime-gpu.cdx.json` |
| zenith-scheduler   | `sbom/zenith-scheduler.cdx.json`   | 
| zenith-dataplane   | `sbom/zenith-dataplane.cdx.json`   |
| + 8 more modules   | `sbom/*.cdx.json`                  |

## Generation Process

SBOMs are automatically generated:

1. **On every release** via GitHub Actions
2. **On demand** via `cargo cyclonedx`

### Manual Generation

```bash
# Generate SBOM for all packages
cargo cyclonedx --format json --spec-version 1.5 --all

# Find generated files
find . -name "*.cdx.json" -not -path "./target/*"
```

## SBOM Contents

Each SBOM includes:

- **Metadata**: Timestamp, tool version, component info
- **Dependencies**: All direct and transitive dependencies
- **Licenses**: SPDX license identifiers
- **PURLs**: Package URLs for all components
- **Hashes**: SHA-256 checksums where available

## Verification

### Verify SBOM Integrity

```bash
# Calculate SBOM hash (matches release notes)
sha256sum sbom/*.cdx.json | sha256sum
```

### Scan for Vulnerabilities

```bash
# Using Grype
grype sbom:sbom/zenith-core.cdx.json

# Using Trivy
trivy sbom sbom/zenith-core.cdx.json
```

## SLSA Compliance

Our SBOM policy supports **SLSA Level 4**:

| Requirement      | Implementation            |
|------------------|---------------------------|
| Source integrity | Git with signed commits   |
| Build integrity  | GitHub Actions (hermetic) |
| Provenance       | SLSA attestation          |
| Dependencies     | SBOM with all deps        |

## Dependency Updates

- **Weekly**: Dependabot scans
- **Monthly**: Full audit review
- **On CVE**: Immediate response

## Contact

Security issues: security@zenith-project.io

---

*This policy is reviewed quarterly.*
