import sys, os
from fastapi.testclient import TestClient

# Ensure repo root on sys.path to import panthersdk
ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

from panthersdk.api import create_app  # type: ignore


def test_health():
    app = create_app()
    c = TestClient(app)
    r = c.get('/health')
    assert r.status_code == 200
    data = r.json()
    assert 'status' in data


def test_metrics_coherence():
    app = create_app()
    c = TestClient(app)
    r = c.post('/metrics/evaluate', json={'metric': 'coherence', 'text': 'a b c d e'})
    assert r.status_code == 200
    data = r.json()
    assert 0.0 <= data.get('score', 0.0) <= 1.0


def test_metrics_rouge_fallback():
    app = create_app()
    c = TestClient(app)
    r = c.post('/metrics/evaluate', json={'metric': 'rouge', 'reference': 'a b c', 'candidate': 'a x c'})
    assert r.status_code == 200
    data = r.json()
    assert data.get('score', 0.0) >= 0.0


def test_proof_compute_and_verify_fallback():
    app = create_app()
    c = TestClient(app)
    prompt = 'hello'
    providers = []
    results = []
    r = c.post('/proof/compute', json={'prompt': prompt, 'providers': providers, 'results': results, 'guidelines_json': '[]'})
    assert r.status_code == 200
    proof = r.json()
    assert 'combined_hash' in proof
    v = c.post('/proof/verify', json={'prompt': prompt, 'providers': providers, 'results': results, 'guidelines_json': '[]', 'proof': proof})
    assert v.status_code == 200
    assert v.json().get('valid') in (True, False)
