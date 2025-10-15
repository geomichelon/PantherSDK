# PantherSDK API (Python)

This repository contains a Python API service (FastAPI) that exposes the PantherSDK core over HTTP for mobile/web and enterprise integrations.

Location of the code:
- Python package source: `python/panthersdk`
- Entrypoint to run locally: from `python/` → `uvicorn panthersdk.api:create_app --host 0.0.0.0 --port 8000`

Why this folder?
- To make it obvious that the Python code in this repo is the API service. We keep the canonical code and packaging under `python/` (pyproject, wheel builds, CI paths) to avoid breaking tooling.

If you want to actually rename `python/` → `api/` in the future, plan to update:
- CI: `.github/workflows/ci.yml` working-directory currently points to `python`
- Build scripts: `scripts/release/build_python_wheel.sh`
- Docs: references to `python/panthersdk` in `README.md` and guides
- Docker and local scripts referencing the `python/` path

For now, use this folder as a discoverability alias; the source of truth remains `python/panthersdk`.
