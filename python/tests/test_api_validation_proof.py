import sys, os, json
from fastapi.testclient import TestClient

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

from panthersdk.api import create_app  # type: ignore


def test_validation_run_multi_without_ffi_returns_error():
    app = create_app()
    c = TestClient(app)
    providers = []
    r = c.post('/validation/run_multi', json={'prompt': 'hi', 'providers': providers})
    assert r.status_code == 200
    data = r.json()
    assert 'error' in data


def test_proof_history_empty_list():
    app = create_app()
    c = TestClient(app)
    r = c.get('/proof/history')
    assert r.status_code == 200
    data = r.json()
    assert isinstance(data, list)

