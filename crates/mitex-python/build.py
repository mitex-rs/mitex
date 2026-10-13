#!/usr/bin/env python3
import argparse
import subprocess
import sys
import shutil
from pathlib import Path

def run_command(cmd, cwd=None):
    print(f"Running: {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=cwd)
    if result.returncode != 0:
        sys.exit(result.returncode)

def ensure_maturin():
    if shutil.which("maturin") is None:
        print("Error: maturin not found. Install it with:")
        print("  python3 -m pip install maturin")
        sys.exit(1)

def main():
    parser = argparse.ArgumentParser(description="Build mitex-python")
    parser.add_argument("--release", action="store_true", help="Build in release mode")
    parser.add_argument("--wheel", action="store_true", help="Build wheel for distribution")
    parser.add_argument("--test", action="store_true", help="Run tests after building")
    args = parser.parse_args()

    crate_path = Path(__file__).parent
    project_root = crate_path.parent.parent  # repo root
    build_opts = ["--release"] if args.release else []

    ensure_maturin()

    if args.wheel:
        run_command(["maturin", "build", *build_opts], cwd=crate_path)
        print(f"Wheel(s) in: {project_root / 'target' / 'wheels'}")
    else:
        run_command(["maturin", "develop", *build_opts], cwd=crate_path)

    if args.test:
        try:
            import pytest  # noqa: F401
        except ImportError:
            print("Installing pytest...")
            run_command([sys.executable, "-m", "pip", "install", "pytest"])

        run_command([sys.executable, "-m", "pytest", str(crate_path / "test.py"), "-v"])

if __name__ == "__main__":
    main()