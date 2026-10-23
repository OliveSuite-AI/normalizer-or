"""Release versions must agree across source, native binding, and distribution."""

from email.message import Message
import io
from pathlib import Path
import shutil
import tarfile
import tomllib
import zipfile
import urllib.error

import pytest
from scripts import release
from scripts.release import choose_version, prepare, verify_distributions


@pytest.mark.parametrize(
    "base,releases,expected",
    [
        ("0.1.0", [], "0.1.0"),
        ("0.1.0", ["0.1.0"], "0.1.1"),
        ("0.1.0", ["0.1.9", "0.1.2"], "0.1.10"),
        ("0.2.0", ["0.1.9"], "0.2.0"),
        ("0.1.5", ["0.1.2"], "0.1.5"),
        ("0.1.0", ["0.1.4", "0.1.8rc1", "invalid"], "0.1.5"),
    ],
)
def test_choose_version(base, releases, expected):
    assert choose_version(base, releases) == expected


def test_reject_old_series_and_invalid_base():
    with pytest.raises(ValueError):
        choose_version("0.1.0", ["0.2.0"])
    with pytest.raises(ValueError):
        choose_version("0.1.0rc1", [])


@pytest.mark.parametrize("status", [404, 403, 500])
def test_registry_errors_do_not_reuse_a_published_version(monkeypatch, status):
    def fail(*args, **kwargs):
        raise urllib.error.HTTPError("https://pypi.org", status, "fixture", {}, None)

    monkeypatch.setattr(release.urllib.request, "urlopen", fail)
    if status == 404:
        assert (
            release.select_version("pypi")
            == tomllib.loads(
                (release.ROOT / "bindings/python/pyproject.toml").read_text()
            )["project"]["version"]
        )
    else:
        with pytest.raises(urllib.error.HTTPError):
            release.select_version("pypi")


def copy_manifests(destination):
    root = Path(__file__).resolve().parents[3]
    for name in [
        "Cargo.toml",
        "Cargo.lock",
        "bindings/python/Cargo.toml",
        "bindings/python/pyproject.toml",
    ]:
        path = destination / name
        path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(root / name, path)


def test_prepare_updates_owned_versions_only_and_is_idempotent(tmp_path):
    copy_manifests(tmp_path)
    lock = tomllib.loads((tmp_path / "Cargo.lock").read_text())
    dependencies = [
        p
        for p in lock["package"]
        if p["name"] not in ("normalizer-or", "normalizer-or-python")
    ]
    prepare(tmp_path, "0.1.123")
    for name, section in [
        ("Cargo.toml", "package"),
        ("bindings/python/Cargo.toml", "package"),
        ("bindings/python/pyproject.toml", "project"),
    ]:
        assert (
            tomllib.loads((tmp_path / name).read_text())[section]["version"]
            == "0.1.123"
        )
    native = tomllib.loads((tmp_path / "bindings/python/Cargo.toml").read_text())
    assert native["dependencies"]["normalizer-or"]["version"] == "=0.1.123"
    updated = tomllib.loads((tmp_path / "Cargo.lock").read_text())["package"]
    assert [
        p for p in updated if p["name"] not in ("normalizer-or", "normalizer-or-python")
    ] == dependencies
    assert {
        p["version"]
        for p in updated
        if p["name"] in ("normalizer-or", "normalizer-or-python")
    } == {"0.1.123"}
    prepare(tmp_path, "0.1.123")


def test_prepare_validates_before_writing(tmp_path):
    copy_manifests(tmp_path)
    root_manifest = tmp_path / "Cargo.toml"
    before = root_manifest.read_bytes()
    native = tmp_path / "bindings/python/Cargo.toml"
    native.write_text(native.read_text().replace('version = "=0.', 'version = "=9.'))
    with pytest.raises(ValueError):
        prepare(tmp_path, "0.1.123")
    assert root_manifest.read_bytes() == before
    with pytest.raises(ValueError):
        prepare(tmp_path, "0.1.123\nInjected=value")


def fake_distributions(path, linux_tag="manylinux_2_17_x86_64"):
    metadata = Message()
    metadata["Name"] = "normalizer-or"
    metadata["Version"] = "0.1.123"
    metadata["Author"] = "Sambit Sekhar"
    platforms = [linux_tag, "win_amd64", "macosx_11_0_arm64", "macosx_10_13_x86_64"]
    for python in ["cp311", "cp312", "cp313", "cp314"]:
        for platform in platforms:
            with zipfile.ZipFile(
                path / f"normalizer_or-0.1.123-{python}-{python}-{platform}.whl", "w"
            ) as z:
                z.writestr(
                    "normalizer_or-0.1.123.dist-info/METADATA", metadata.as_bytes()
                )
                z.writestr(
                    "normalizer_or-0.1.123.dist-info/licenses/LICENSE", "test fixture"
                )
                z.writestr("normalizer_or/py.typed", "")
                z.writestr("normalizer_or/_native.pyi", "")
    with tarfile.open(path / "normalizer_or-0.1.123.tar.gz", "w:gz") as t:
        entry = tarfile.TarInfo("normalizer_or-0.1.123/PKG-INFO")
        entry.size = len(metadata.as_bytes())
        t.addfile(entry, io.BytesIO(metadata.as_bytes()))


def test_require_complete_portable_matrix(tmp_path):
    fake_distributions(tmp_path)
    verify_distributions(tmp_path, "0.1.123", 16)
    next(tmp_path.glob("*cp314*win_amd64.whl")).unlink()
    with pytest.raises(ValueError):
        verify_distributions(tmp_path, "0.1.123", 16)


def test_reject_native_linux_wheels(tmp_path):
    fake_distributions(tmp_path, "linux_x86_64")
    with pytest.raises(ValueError, match="Nonportable"):
        verify_distributions(tmp_path, "0.1.123", 16)
