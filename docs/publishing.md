# Publishing the Python package

The distribution name is `normalizer-or`; Python code imports `normalizer_or`.
It supports CPython 3.11–3.14. Both pip and uv install the same PyPI distributions;
there is no separate uv registry or package format.

## Current status

The maturin backend already builds an sdist and native wheels. Local tests install
the exact built wheel. CI tests Windows, Linux, and macOS with each supported
Python version. `.github/workflows/release.yml` currently builds artifacts only,
on manual dispatch: it neither publishes to a registry nor creates a GitHub release.
Its single Ubuntu wheel is not a portable release matrix.

The PyPI JSON endpoint for `normalizer-or` returned HTTP 404 on 2026-10-24.
That does not reserve the name or guarantee PyPI will accept it. No package has
been published by this repository's automation.

## Install before publication

Install from this repository, with Git and the pinned Rust toolchain available:

```sh
python -m pip install "git+https://github.com/OliveSuite-AI/normalizer-or.git#subdirectory=bindings/python"
```

Or use uv with a virtual environment:

```sh
uv venv
uv pip install "git+https://github.com/OliveSuite-AI/normalizer-or.git#subdirectory=bindings/python"
```

For reproducibility, insert `@<commit-sha>` immediately before `#subdirectory`.
The root is a Cargo project; the Python build project is `bindings/python`.

## Recommended PyPI release setup

1. Complete native-speaker review of the alpha linguistic defaults before a
   production release. Passing text tests does not establish speech quality.
2. Extend `release.yml` to build CPython 3.11–3.14 wheels for Linux x86_64,
   Windows x86_64, and macOS arm64/x86_64. Existing CI uses `macos-14` without
   an explicit architecture matrix; add native validation for both architectures.
   Use a manylinux container or maturin's Zig support for portable Linux wheels;
   an ordinary Ubuntu `linux_*` wheel is unsuitable for PyPI. Build one sdist
   and verify wheels can be built from that sdist. Add other architectures only
   when their wheels can be tested.
3. Install and test every exact wheel, validate distribution metadata, check
   version agreement across both Cargo manifests and `pyproject.toml`, and
   aggregate only the validated wheels and sdist into a publication artifact.
   Use a unique version for each upload; PyPI does not allow replacing files.
4. Create PyPI and TestPyPI accounts with two-factor authentication. Configure a
   **pending Trusted Publisher** on each service for the first upload:

   | Field | PyPI | TestPyPI |
   |---|---|---|
   | Project name | `normalizer-or` | `normalizer-or` |
   | Owner | `OliveSuite-AI` | `OliveSuite-AI` |
   | Repository | `normalizer-or` | `normalizer-or` |
   | Workflow filename | `release.yml` | `release.yml` |
   | Environment | `pypi` | `testpypi` |

5. Add matching GitHub environments and separate publishing jobs to `release.yml`.
   Each job downloads the validated distribution artifact, runs on Linux, grants
   `id-token: write`, and uses `pypa/gh-action-pypi-publish` pinned to a reviewed
   commit. TestPyPI needs `repository-url: https://test.pypi.org/legacy/`.
   Trusted Publishing avoids storing a long-lived PyPI API token. Keep the
   existing build-only dispatch available for rehearsals. Gate publication on an
   explicit release action and matching version, rather than every push to main.
6. Publish to TestPyPI first, then install in a fresh environment and run a smoke
   check. With compatible wheels uploaded, use:

   ```sh
   python -m pip install --only-binary=:all: --index-url https://test.pypi.org/simple/ normalizer-or==0.1.0
   # In a separate fresh virtual environment:
   uv pip install --only-binary=:all: --index-url https://test.pypi.org/simple/ normalizer-or==0.1.0
   ```

7. Publish the validated distributions to PyPI, then verify the commands below
   against PyPI. Document the exact wheel platforms actually uploaded.

Publisher registration requires access to the PyPI account that will own the
project. GitHub repository access alone cannot create that account-side trust.
The publishing jobs described above are remaining work, not currently enabled.

## Install after publication

These commands require a successful PyPI upload first:

```sh
python -m pip install normalizer-or
uv pip install normalizer-or
# Or add a dependency to a uv-managed application:
uv add normalizer-or
```

Smoke check:

```sh
python -c 'from normalizer_or import Normalizer; assert Normalizer().normalize("୨୫ kg").normalized_text == "ପଚିଶ କିଲୋଗ୍ରାମ"'
```

Compatible wheels avoid a Rust compiler on the user's machine. If no compatible
wheel exists, source installation requires Rust and build tooling. The declared
Python range still applies even when an sdist is available.

## References

- [PyPI: create a project with a Trusted Publisher](https://docs.pypi.org/trusted-publishers/creating-a-project-through-oidc/)
- [PyPI: publishing from GitHub Actions](https://docs.pypi.org/trusted-publishers/using-a-publisher/)
- [maturin: distribution and manylinux requirements](https://www.maturin.rs/distribution.html)
- [uv: installing packages](https://docs.astral.sh/uv/pip/packages/)
