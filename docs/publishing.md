# Python releases

The distribution is `normalizer-or`; Python imports `normalizer_or`. pip and uv
use the same PyPI files. Publishing to PyPI also makes `uv add normalizer-or`
available without a separate registry.

## Automatic releases after merging a PR

`.github/workflows/release.yml` waits for **Odia CI** to finish successfully on a
push to `main`. It verifies that the exact commit belongs to a merged PR targeting
`main` and is still the latest commit on `main`. Failed CI, direct pushes and
closed/unmerged PRs do not publish. Superseded commits are skipped so delayed CI
cannot publish older code as a newer release.

The workflow then:

1. Reads the registry and selects the next unused patch version in the source's
   major/minor series. With source `0.1.0`, the first upload is `0.1.0`, followed by
   `0.1.1`, `0.1.2`, and so on. Every merged PR, including documentation changes,
   is eligible. Increase the source major/minor version deliberately for a new series.
2. Applies that version to both Cargo manifests, the Python manifest, the binding's
   exact core dependency and the two owned Cargo.lock entries **in the build checkout**.
   Source control is not edited or committed back to `main`. The installed
   `BUILD_VERSION` and `NORMALIZER_ID` report the allocated release version.
3. Builds **16 wheels** for CPython 3.11–3.14: Linux x86_64, Windows x86_64,
   macOS arm64 and macOS x86_64. Linux builds use maturin's Zig support with
   `manylinux_2_17`. Each wheel is installed and runs the Python tests and
   independent Rust/Python conformance checks on its native runner.
4. Builds one sdist, builds a wheel from that sdist, and tests it plus the Rust core.
5. Checks the complete wheel matrix, package/version/author identity, typing and
   license assets, then runs `twine check --strict` on the final distributions.
6. Publishes only those validated artifacts using OIDC Trusted Publishing.

Release workflows are serialized; active uploads are never cancelled. GitHub
keeps at most one pending run in a concurrency group, so rapid merges can
supersede intermediate pending releases. The latest passing merged commit remains
eligible. There is no promise of a separate release for every intermediate commit.

## One-time account setup

Create/sign in to your PyPI and TestPyPI accounts and enable two-factor
authentication. Register a **pending Trusted Publisher** if the project does not
exist yet, or add a publisher to the existing project you own:

| Field | PyPI | TestPyPI |
|---|---|---|
| Project name | `normalizer-or` | `normalizer-or` |
| GitHub owner | `OliveSuite-AI` | `OliveSuite-AI` |
| Repository | `normalizer-or` | `normalizer-or` |
| Workflow filename | `release.yml` | `release.yml` |
| GitHub environment | `pypi` | `testpypi` |

Register at [PyPI publishing settings](https://pypi.org/manage/account/publishing/)
and [TestPyPI publishing settings](https://test.pypi.org/manage/account/publishing/).
Configuring a pending publisher does not reserve the package name. The project's
PyPI JSON endpoint returned 404 during the initial readiness check.

The workflow's publish job uses the corresponding GitHub environment and
`id-token: write`. No `PYPI_API_TOKEN` secret is required. If you add required
reviewers to an environment, publication waits for their approval. Without
required reviewers it publishes automatically after all checks pass.

**GitHub access alone cannot register the PyPI account-side publisher.** Until
that registration is complete, publication fails at authentication even when
builds pass. Do not put a PyPI password or token in repository files.

## First run and recovery

In GitHub Actions, select **Publish Python package → Run workflow**, on `main`:

| Destination | Behavior |
|---|---|
| `build` — default | Build/test/validate and retain artifacts; no upload |
| `testpypi` | Select a TestPyPI version and publish to the test registry |
| `pypi` | Select a PyPI version and publish the current `main` commit |

Manual runs provide first-release bootstrapping and recovery; they execute their
own distribution tests but do not require a completed CI event. Rehearse with
`build`, then `testpypi` before the first production upload. The complete validated
artifact is named `normalizer-or-distributions`.

If an upload partially fails, rerun the **failed publish job** to reuse the same
validated artifacts; `skip-existing` allows already-uploaded files. Rerunning all
jobs selects another unused patch version. PyPI versions/files cannot be replaced.
Registry/network errors other than a missing project stop version allocation;
release automation never guesses that the package is absent after an API failure.

For supported platforms, test the TestPyPI upload in a fresh virtual environment:

```sh
python -m pip install --only-binary=:all: --index-url https://test.pypi.org/simple/ normalizer-or
# Or:
uv pip install --only-binary=:all: --index-url https://test.pypi.org/simple/ normalizer-or
```

After successful publication to PyPI:

```sh
pip install normalizer-or
uv pip install normalizer-or
uv add normalizer-or
python -c 'from normalizer_or import Normalizer; assert Normalizer().normalize("୨୫ kg").normalized_text == "ପଚିଶ କିଲୋଗ୍ରାମ"'
```

Until the first upload, source installation remains available:

```sh
python -m pip install "git+https://github.com/OliveSuite-AI/normalizer-or.git#subdirectory=bindings/python"
```

Compatible wheels avoid a Rust compiler on the user's machine. Source builds
need Rust; unsupported platforms may build from the sdist. This workflow does not
publish the Rust crate to crates.io. Linguistic defaults remain alpha until
native-speaker/audio review is complete.

## References

- [PyPI: configure a new Trusted Publisher](https://docs.pypi.org/trusted-publishers/creating-a-project-through-oidc/)
- [PyPI: publish from GitHub Actions](https://docs.pypi.org/trusted-publishers/using-a-publisher/)
- [maturin: portable distributions and Zig](https://www.maturin.rs/distribution.html)
