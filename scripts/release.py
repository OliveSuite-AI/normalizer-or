"""Select a registry version, prepare consistent manifests, and verify distributions."""

import argparse
from email.parser import BytesParser
import json
import os
from pathlib import Path
import re
import tarfile
import tomllib
import urllib.error
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[1]
VERSION = re.compile(r"(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)")
REGISTRIES = {"pypi": "https://pypi.org", "testpypi": "https://test.pypi.org"}


def version_tuple(value):
    if not VERSION.fullmatch(value):
        raise ValueError(f"Expected MAJOR.MINOR.PATCH, got {value!r}")
    return tuple(map(int, value.split(".")))


def choose_version(base, releases):
    """Keep the source minor series; allocate a fresh final patch version."""
    current = version_tuple(base)
    existing = [version_tuple(v) for v in releases if VERSION.fullmatch(v)]
    if any(v[:2] > current[:2] for v in existing):
        raise ValueError("Source version series is older than a published release")
    patches = [v[2] for v in existing if v[:2] == current[:2]]
    patch = max(current[2], max(patches) + 1) if patches else current[2]
    return f"{current[0]}.{current[1]}.{patch}"


def select_version(registry):
    base = tomllib.loads((ROOT / "bindings/python/pyproject.toml").read_text())[
        "project"
    ]["version"]
    request = urllib.request.Request(
        REGISTRIES[registry] + "/pypi/normalizer-or/json",
        headers={"User-Agent": "normalizer-or-release"},
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            releases = json.load(response)["releases"]
    except urllib.error.HTTPError as exc:
        if exc.code != 404:
            raise
        releases = {}
    return choose_version(base, releases)


def replace_section(text, heading, transform):
    pattern = re.compile(
        r"(^" + re.escape(heading) + r"\n)(.*?)(?=^\[|\Z)", re.M | re.S
    )
    result, count = pattern.subn(lambda m: m[1] + transform(m[2]), text)
    if count != 1:
        raise ValueError(f"Expected one {heading} section")
    return result


def replace_version(section, version):
    result, count = re.subn(
        r'^version = "[^"]+"$', f'version = "{version}"', section, flags=re.M
    )
    if count != 1:
        raise ValueError("Expected exactly one version field")
    return result


def prepare(root, version):
    version_tuple(version)
    files = [
        root / "Cargo.toml",
        root / "bindings/python/Cargo.toml",
        root / "bindings/python/pyproject.toml",
    ]
    contents = [p.read_text(encoding="utf-8") for p in files]
    parsed = [tomllib.loads(text) for text in contents]
    base = parsed[0]["package"]["version"]
    if (
        parsed[1]["package"]["version"] != base
        or parsed[2]["project"]["version"] != base
    ):
        raise ValueError("Source manifest versions disagree")
    if parsed[1]["dependencies"]["normalizer-or"]["version"] != "=" + base:
        raise ValueError("Binding dependency version disagrees")
    updated = [
        replace_section(text, heading, lambda s: replace_version(s, version))
        for text, heading in zip(contents, ["[package]", "[package]", "[project]"])
    ]
    updated[1] = updated[1].replace(f'version = "={base}"', f'version = "={version}"')
    lock_path = root / "Cargo.lock"
    lock = lock_path.read_text(encoding="utf-8")
    owned = {"normalizer-or", "normalizer-or-python"}
    seen = set()

    def update_package(match):
        section = match[0]
        package = tomllib.loads(section)["package"][0]
        if package["name"] in owned:
            if package["version"] != base:
                raise ValueError("Lockfile version disagrees")
            seen.add(package["name"])
            return replace_version(section, version)
        return section

    lock = re.sub(
        r"^\[\[package\]\]\n.*?(?=^\[\[package\]\]|\Z)",
        update_package,
        lock,
        flags=re.M | re.S,
    )
    if seen != owned:
        raise ValueError("Missing owned package in lockfile")
    # Validate every edit before writing any file.
    for text in [*updated, lock]:
        tomllib.loads(text)
    for path, text in zip([*files, lock_path], [*updated, lock]):
        path.write_text(text, encoding="utf-8")


def verify_distributions(directory, version, wheel_count):
    wheels = sorted(directory.glob("*.whl"))
    sdists = sorted(directory.glob("*.tar.gz"))
    if len(wheels) != wheel_count or len(sdists) != 1:
        raise ValueError(
            f"Expected {wheel_count} wheels and one sdist; found {len(wheels)}, {len(sdists)}"
        )
    for wheel in wheels:
        with zipfile.ZipFile(wheel) as archive:
            names = archive.namelist()
            metadata = BytesParser().parsebytes(
                archive.read(next(n for n in names if n.endswith("/METADATA")))
            )
            assert (
                metadata["Name"] == "normalizer-or" and metadata["Version"] == version
            )
            assert metadata["Author"] == "Sambit Sekhar"
            assert (
                "normalizer_or/py.typed" in names
                and "normalizer_or/_native.pyi" in names
            )
            assert any("/licenses/" in n for n in names)
        if "linux_" in wheel.name and "manylinux_2_17" not in wheel.name:
            raise ValueError(f"Nonportable Linux wheel: {wheel.name}")
    if wheel_count == 16:
        platforms = {
            "Linux x64": r"manylinux_2_17_x86_64",
            "Windows x64": r"win_amd64",
            "macOS arm64": r"macosx_\d+_\d+_arm64",
            "macOS x64": r"macosx_\d+_\d+_x86_64",
        }
        for python in ["cp311", "cp312", "cp313", "cp314"]:
            for platform, pattern in platforms.items():
                matches = [
                    w
                    for w in wheels
                    if f"-{python}-" in w.name and re.search(pattern, w.name)
                ]
                if len(matches) != 1:
                    raise ValueError(
                        f"Missing/duplicate wheel for {python}, {platform}"
                    )
    with tarfile.open(sdists[0]) as archive:
        member = next(m for m in archive.getmembers() if m.name.endswith("/PKG-INFO"))
        metadata = BytesParser().parsebytes(archive.extractfile(member).read())
        assert metadata["Name"] == "normalizer-or" and metadata["Version"] == version
    print(f"Verified {len(wheels)} wheels and one sdist for {version}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    select = sub.add_parser("select")
    select.add_argument("--registry", choices=REGISTRIES, default="pypi")
    stage = sub.add_parser("prepare")
    stage.add_argument("version")
    verify = sub.add_parser("verify")
    verify.add_argument("directory", type=Path)
    verify.add_argument("version")
    verify.add_argument("--wheels", type=int, default=16)
    args = parser.parse_args()
    if args.command == "select":
        version = select_version(args.registry)
        print(version)
        if output := os.environ.get("GITHUB_OUTPUT"):
            with open(output, "a", encoding="utf-8") as stream:
                stream.write(f"version={version}\n")
    elif args.command == "prepare":
        prepare(ROOT, args.version)
    else:
        verify_distributions(args.directory, args.version, args.wheels)


if __name__ == "__main__":
    main()
