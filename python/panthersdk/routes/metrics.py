from fastapi import APIRouter, Depends, HTTPException, Response
from pydantic import BaseModel
import json
from ..core import get_rust, auth_guard

router = APIRouter()

try:
    from prometheus_client import Counter, Histogram, generate_latest, CONTENT_TYPE_LATEST
    _PROM = True
except Exception:
    _PROM = False
    Counter = Histogram = None  # type: ignore

if _PROM:
    MET_VAL_REQ = Counter('panther_validation_requests_total', 'Total validation requests')
    MET_VAL_ERR = Counter('panther_validation_errors_total', 'Total validation errors')
    MET_VAL_ERR_L = Counter('panther_validation_errors_labeled_total', 'Validation errors by provider/category', ['provider','category'])
    MET_VAL_LAT = Histogram('panther_validation_latency_seconds', 'Validation latency (seconds)')


class EvaluateRequest(BaseModel):
    metric: str
    expected: str | None = None
    generated: str | None = None
    reference: str | None = None
    candidate: str | None = None
    text: str | None = None
    samples: list[str] | None = None


@router.post("/metrics/evaluate")
def metrics_evaluate(req: EvaluateRequest, _auth=Depends(auth_guard)):
    m = req.metric.lower()
    lib = get_rust()
    if m == "bleu" and req.reference and req.candidate:
        try:
            import pantherpy
            return {"score": pantherpy.evaluate_bleu_py(req.reference, req.candidate)}
        except Exception:
            pass
    if m == "accuracy" and req.expected is not None and req.generated is not None:
        et = req.expected.split(); gt = req.generated.split()
        n = max(len(et), len(gt)) or 1
        matches = sum(1 for a, b in zip(et, gt) if a == b)
        return {"score": matches / n}
    if m == "coherence" and req.text is not None:
        t = req.text.split();
        if len(t) < 2: return {"score": 1.0}
        bigrams = set(); repeats = 0
        for i in range(len(t) - 1):
            bg = (t[i], t[i + 1])
            repeats += 1 if bg in bigrams else 0
            bigrams.add(bg)
        return {"score": 1.0 - repeats / max(1, len(t) - 1)}
    if m == "diversity" and req.samples is not None:
        tokens = []; types = set()
        for s in req.samples:
            for tok in s.split(): tokens.append(tok); types.add(tok)
        if not tokens: return {"score": 0.0}
        return {"score": len(types) / len(tokens)}
    if m == "fluency" and req.text is not None:
        toks = req.text.split();
        if not toks: return {"score": 0.0}
        vowels = set("aeiouAEIOU"); good = sum(1 for t in toks if any(ch in vowels for ch in t))
        return {"score": good / len(toks)}
    if m == "rouge" and req.reference and req.candidate:
        if lib and hasattr(lib, "panther_metrics_rouge_l"):
            s = lib.panther_metrics_rouge_l(req.reference.encode("utf-8"), req.candidate.encode("utf-8"))
            return {"score": float(s)}
        # fallback LCS
        def _tok(s: str): return s.split()
        r = _tok(req.reference); c = _tok(req.candidate)
        if not r or not c: return {"score": 0.0}
        n, m = len(r), len(c)
        dp = [[0]*(m+1) for _ in range(n+1)]
        for i in range(n):
            for j in range(m):
                dp[i+1][j+1] = dp[i][j]+1 if r[i]==c[j] else max(dp[i+1][j], dp[i][j+1])
        lcs = dp[n][m]; p = lcs/len(c); rc = lcs/len(r); s = (2*p*rc)/(p+rc) if p+rc else 0.0
        return {"score": s}
    if m == "factcheck" and req.text is not None and req.samples is not None:
        facts = req.samples
        if lib and hasattr(lib, "panther_metrics_fact_coverage"):
            s = lib.panther_metrics_fact_coverage(json.dumps(facts).encode("utf-8"), req.text.encode("utf-8"))
            return {"score": float(s)}
        low = req.text.lower(); hits = sum(1 for f in facts if f and f.lower() in low); tot = max(1,len(facts))
        return {"score": hits / tot}
    if m in ("factcheck-adv","factcheck_adv") and req.text is not None and req.samples is not None:
        facts = req.samples
        if lib and hasattr(lib, "panther_metrics_factcheck_adv"):
            s = lib.panther_metrics_factcheck_adv(json.dumps(facts).encode("utf-8"), req.text.encode("utf-8"))
            return {"score": float(s)}
        # Fallback: heuristic contradiction penalty near facts
        neg = {"not","no","never","without","contraindicated","avoid"}
        toks = req.text.lower().split()
        def coverage(fs: list[str]) -> float:
            low = req.text.lower();
            hits = sum(1 for f in fs if f and f.lower() in low); tot = max(1,len(fs)); return hits/tot
        def contradictions(fs: list[str]) -> float:
            c = 0
            for f in fs:
                if not f: continue
                f0 = f.split()[0].lower()
                for i,t in enumerate(toks):
                    if t == f0:
                        w = toks[max(0,i-3):min(len(toks),i+3)]
                        if any(x in neg for x in w): c += 1; break
            return c/ max(1,len(fs))
        cov = coverage(facts); con = contradictions(facts)
        score = max(0.0, min(1.0, cov * (1.0 - 0.7*con)))
        return {"score": score, "coverage": cov, "contradiction_rate": con}
    if m == "plagiarism" and req.text is not None and req.samples is not None:
        lib = get_rust()
        if lib and hasattr(lib, "panther_metrics_plagiarism"):
            try:
                import json as _j
                s = lib.panther_metrics_plagiarism(_j.dumps(req.samples).encode("utf-8"), req.text.encode("utf-8"))
                return {"score": float(s)}
            except Exception:
                pass
        # Fallback: Jaccard of 3-grams
        import re
        def norm(t: str) -> list[str]:
            t = t.lower()
            t = re.sub(r"[^a-z0-9\s]", " ", t)
            return [x for x in t.split() if x]
        def ngrams(ws: list[str], n: int = 3) -> set[str]:
            if len(ws) < n: return set()
            return {" ".join(ws[i:i+n]) for i in range(0, len(ws)-n+1)}
        cand = ngrams(norm(req.text), 3)
        if not cand: return {"score": 0.0}
        best = 0.0
        for doc in req.samples:
            gs = ngrams(norm(doc), 3)
            if not gs: continue
            inter = len(cand & gs); uni = len(cand | gs)
            s = (inter / uni) if uni else 0.0
            if s > best: best = s
        return {"score": best}
    return {"error": "unsupported or missing fields"}


@router.get("/metrics")
def metrics_exporter():
    if not _PROM:
        raise HTTPException(status_code=503, detail="prometheus not enabled")
    return Response(content=generate_latest(), media_type=CONTENT_TYPE_LATEST)  # type: ignore
