from fastapi import APIRouter, Depends, HTTPException, Response
from pydantic import BaseModel
import json
from ..core import get_rust, auth_guard, db_insert_factcheck_audit, db_insert_rewrite_audit
import os

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
    ngram: int | None = None
    # For advanced fact-checking with sources
    sources: list[dict] | None = None  # each: {id: str, text: str}
    # For advanced bias with custom groups
    groups: dict[str, list[str]] | None = None
    # Advanced fact-checking options
    top_k: int | None = None
    evidence_k: int | None = None
    method: str | None = None  # for factcheck_sources: jaccard|bow
    contradiction_terms: list[str] | None = None
    contradiction_method: str | None = None  # heuristic|nli
    locale: str | None = None  # for bias groups defaults (en|pt)
    similarity_threshold: float | None = None  # filter low-similarity sources/evidence
    domain: str | None = None  # for bias dictionaries (e.g., healthcare, finance)
    neutralize: bool | None = None  # if true, return neutralized samples suggestions
    source_thresholds: dict[str, float] | None = None  # per-source acceptance thresholds
    # Contextual relevance by domain/locale
    keywords_must: list[str] | None = None
    keywords_should: list[str] | None = None
    rewrite_method: str | None = None  # rule|llm
    target_style: str | None = None  # neutral|formal|simple


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
        if lib and hasattr(lib, "panther_metrics_plagiarism_ngram") and isinstance(req.ngram, int):
            try:
                import json as _j
                n = int(req.ngram) if (req.ngram or 0) > 0 else 3
                s = lib.panther_metrics_plagiarism_ngram(_j.dumps(req.samples).encode("utf-8"), req.text.encode("utf-8"), n)
                return {"score": float(s)}
            except Exception:
                pass
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
        n = int(req.ngram) if (req.ngram or 0) > 0 else 3
        cand = ngrams(norm(req.text), n)
        if not cand: return {"score": 0.0}
        best = 0.0
        for doc in req.samples:
            gs = ngrams(norm(doc), n)
            if not gs: continue
            inter = len(cand & gs); uni = len(cand | gs)
            s = (inter / uni) if uni else 0.0
            if s > best: best = s
        return {"score": best}

    if m in ("factcheck_sources", "factcheck-sources") and req.text is not None and req.sources is not None:
        # Advanced fact-checking with sources: similarity vs sources, coverage, contradictions, evidence sentences.
        import re
        text = req.text
        srcs = []
        for s in req.sources:
            sid = str(s.get('id') or '')
            st = str(s.get('text') or '')
            if st.strip():
                srcs.append((sid, st))
        if not srcs:
            return {"error": "no sources"}
        def norm(t: str) -> list[str]:
            t = t.lower()
            t = re.sub(r"[^a-z0-9\s]", " ", t)
            return [x for x in t.split() if x]
        def jaccard(a: set[str], b: set[str]) -> float:
            if not a and not b: return 1.0
            if not a or not b: return 0.0
            return len(a & b) / len(a | b)
        def bow_cosine(a: list[str], b: list[str]) -> float:
            # simple bag-of-words cosine without idf
            from collections import Counter
            ca, cb = Counter(a), Counter(b)
            keys = set(ca.keys()) | set(cb.keys())
            if not keys: return 0.0
            dot = sum(ca[k]*cb[k] for k in keys)
            na = sum(v*v for v in ca.values()) ** 0.5
            nb = sum(v*v for v in cb.values()) ** 0.5
            if na == 0.0 or nb == 0.0: return 0.0
            return dot / (na*nb)
        cand_tokens = set(norm(text))
        cand_list = norm(text)
        method = (req.method or "jaccard").lower()
        scored = []
        for sid, st in srcs:
            toks = norm(st)
            if method == "bow":
                score = bow_cosine(cand_list, toks)
            else:
                score = jaccard(cand_tokens, set(toks))
            scored.append({"id": sid, "score": score})

        # Optional: embedding-based similarity (OpenAI/Ollama) when method=embed and env allows
        if method == "embed":
            try:
                import httpx  # type: ignore
                # simple cache to avoid duplicate calls
                _EMB_CACHE: dict[str, list[float]] = {}
                def _cos(a: list[float], b: list[float]) -> float:
                    import math
                    da = sum(x*x for x in a) ** 0.5
                    db = sum(x*x for x in b) ** 0.5
                    if da == 0.0 or db == 0.0: return 0.0
                    return sum(x*y for x,y in zip(a,b)) / (da*db)
                def _embed_openai(txts: list[str]) -> list[list[float]]:
                    base = os.getenv("PANTHER_OPENAI_BASE", "https://api.openai.com").rstrip("/")
                    key = os.getenv("PANTHER_OPENAI_API_KEY", "")
                    model = os.getenv("PANTHER_OPENAI_EMBED_MODEL", "text-embedding-3-small")
                    if not key: return []
                    with httpx.Client(timeout=20.0) as client:
                        resp = client.post(f"{base}/v1/embeddings", headers={"Authorization": f"Bearer {key}"}, json={"input": txts, "model": model})
                        if resp.status_code != 200: return []
                        data = resp.json().get("data", [])
                        return [d.get("embedding", []) for d in data]
                def _embed_ollama(txts: list[str]) -> list[list[float]]:
                    base = os.getenv("PANTHER_OLLAMA_BASE", "http://127.0.0.1:11434").rstrip("/")
                    model = os.getenv("PANTHER_OLLAMA_EMBED_MODEL", "nomic-embed-text")
                    outs: list[list[float]] = []
                    with httpx.Client(timeout=20.0) as client:
                        for t in txts:
                            resp = client.post(f"{base}/api/embeddings", json={"model": model, "prompt": t})
                            if resp.status_code != 200: outs.append([])
                            else: outs.append(resp.json().get("embedding", []))
                    return outs
                def _embed_texts(txts: list[str]) -> list[list[float]]:
                    prov = os.getenv("PANTHER_EMBED_PROVIDER", "openai").lower()
                    # return cached or fetch
                    to_fetch: list[str] = []
                    order: list[int] = []
                    for i,t in enumerate(txts):
                        if t in _EMB_CACHE: continue
                        to_fetch.append(t); order.append(i)
                    embs: list[list[float]] = []
                    if to_fetch:
                        embs = _embed_openai(to_fetch) if prov == "openai" else _embed_ollama(to_fetch)
                        for i,e in enumerate(embs):
                            _EMB_CACHE[to_fetch[i]] = e
                    return [_EMB_CACHE.get(t, []) for t in txts]

                # candidate + sources
                cand_emb = _embed_texts([req.text])[0]
                src_embs = _embed_texts([s[1] for s in srcs])
                if cand_emb and all(isinstance(v, float) for v in cand_emb):
                    scored = []
                    for (sid,_), emb in zip(srcs, src_embs):
                        score = _cos(cand_emb, emb) if emb else 0.0
                        scored.append({"id": sid, "score": score})
            except Exception:
                # silently fallback to previous scored list
                pass
        scored.sort(key=lambda x: x['score'], reverse=True)
        k = int(req.top_k) if (req.top_k or 0) > 0 else 3
        sim_th = float(req.similarity_threshold) if (req.similarity_threshold is not None) else 0.0
        topk = [s for s in scored if s['score'] >= sim_th][:k]
        # Coverage as proportion of unique source tokens (from top-k) present in candidate
        top_union: set[str] = set()
        id_set = {t['id'] for t in topk}
        for sid, st in srcs:
            if sid in id_set:
                top_union |= set(norm(st))
        coverage = (len(cand_tokens & top_union) / max(1, len(top_union))) if top_union else 0.0
        # Contradições: heurística padrão, com opção NLI
        neg = set((req.contradiction_terms or []) + ["not","no","never","without","contraindicated","avoid"])
        toks = norm(text)
        contradictions = 0
        keys = list(top_union)[:50]  # limit search
        for kw in keys:
            for i, t in enumerate(toks):
                if t == kw:
                    w = toks[max(0,i-3):min(len(toks), i+3)]
                    if any(x in neg for x in w):
                        contradictions += 1
                        break
        con_rate = contradictions / max(1, len(keys))
        # Overall score (blend): coverage penalized by contradictions
        score = max(0.0, min(1.0, coverage * (1.0 - 0.7*con_rate)))
        # Evidence snippets: top sentences per top source
        ev_k = int(req.evidence_k) if (req.evidence_k or 0) > 0 else 2
        def sent_split(s: str) -> list[str]:
            parts = re.split(r"(?<=[\.!?])\s+", s.strip())
            return [p.strip() for p in parts if p.strip()]
        evidences: list[dict] = []
        cand_set = cand_tokens
        for t in topk:
            sid = t['id']
            st = next((st for (i, st) in srcs if i == sid), '')
            sents = sent_split(st)
            scored_sents = []
            for sent in sents:
                sset = set(norm(sent))
                if method == "embed":
                    try:
                        # use the same embedder for sentence/candidate
                        import httpx  # type: ignore
                        # reuse helper closures
                        # NOTE: in this scope, re-embedding candidate each time may be costly; acceptable for MVP small sents
                        def _cos(a: list[float], b: list[float]) -> float:
                            import math
                            da = sum(x*x for x in a) ** 0.5
                            db = sum(x*x for x in b) ** 0.5
                            if da == 0.0 or db == 0.0: return 0.0
                            return sum(x*y for x,y in zip(a,b)) / (da*db)
                        # basic cache
                        _EMB2: dict[str, list[float]] = {}
                        def _emb_one(txt: str) -> list[float]:
                            if txt in _EMB2: return _EMB2[txt]
                            prov = os.getenv("PANTHER_EMBED_PROVIDER", "openai").lower()
                            if prov == "openai":
                                base = os.getenv("PANTHER_OPENAI_BASE", "https://api.openai.com").rstrip("/")
                                key = os.getenv("PANTHER_OPENAI_API_KEY", "")
                                model = os.getenv("PANTHER_OPENAI_EMBED_MODEL", "text-embedding-3-small")
                                if not key: return []
                                with httpx.Client(timeout=20.0) as client:
                                    resp = client.post(f"{base}/v1/embeddings", headers={"Authorization": f"Bearer {key}"}, json={"input": [txt], "model": model})
                                    if resp.status_code != 200: return []
                                    vec = resp.json().get("data", [{}])[0].get("embedding", [])
                                    _EMB2[txt] = vec; return vec
                            else:
                                base = os.getenv("PANTHER_OLLAMA_BASE", "http://127.0.0.1:11434").rstrip("/")
                                model = os.getenv("PANTHER_OLLAMA_EMBED_MODEL", "nomic-embed-text")
                                with httpx.Client(timeout=20.0) as client:
                                    resp = client.post(f"{base}/api/embeddings", json={"model": model, "prompt": txt})
                                    if resp.status_code != 200: return []
                                    vec = resp.json().get("embedding", [])
                                    _EMB2[txt] = vec; return vec
                        cand_emb = _emb_one(req.text)
                        sent_emb = _emb_one(sent)
                        sscore = _cos(cand_emb, sent_emb) if cand_emb and sent_emb else 0.0
                    except Exception:
                        sscore = 0.0
                else:
                    sscore = bow_cosine(list(cand_set), list(sset)) if method == "bow" else jaccard(cand_set, sset)
                if sscore > 0.0:
                    overlap = sorted(list(cand_set & sset))[:10]
                    # positional highlights (span indices) in candidate text
                    spans = []
                    base = req.text
                    for tok in overlap:
                        start = base.lower().find(tok)
                        if start >= 0:
                            spans.append({"token": tok, "start": start, "end": start + len(tok)})
                    scored_sents.append({"text": sent, "score": sscore, "overlap": overlap, "spans": spans})
            scored_sents.sort(key=lambda x: x['score'], reverse=True)
            evidences.append({"id": sid, "sentences": scored_sents[:ev_k]})
        # Optional NLI classification for contradictions (robust)
        nli_rate = None
        nli_details: list[dict] = []
        if (req.contradiction_method or '').lower() == 'nli':
            try:
                import httpx  # type: ignore
                prov = os.getenv('PANTHER_EMBED_PROVIDER', 'openai').lower()  # reuse provider selection
                # Build pairs candidate<->evidence sentences (top ev_k per source)
                pairs: list[tuple[str,str,str]] = []  # (source_id, src_sentence, candidate)
                for e in evidences:
                    sid = e.get('id')
                    for s in e.get('sentences', [])[:ev_k]:
                        pairs.append((str(sid), str(s.get('text','')), text))
                def classify_openai(pairs):
                    base = os.getenv('PANTHER_OPENAI_BASE','https://api.openai.com').rstrip('/')
                    key = os.getenv('PANTHER_OPENAI_API_KEY','')
                    model = os.getenv('PANTHER_OPENAI_NLI_MODEL','gpt-4o-mini')
                    if not key or not pairs: return []
                    out = []
                    with httpx.Client(timeout=30.0) as client:
                        for sid, premise, hypothesis in pairs:
                            messages = [
                                {"role":"system","content":"You are an NLI classifier. Reply with a single token: entailment, neutral, or contradiction."},
                                {"role":"user","content":f"Premise: {premise}\nHypothesis: {hypothesis}"}
                            ]
                            resp = client.post(f"{base}/v1/chat/completions", headers={"Authorization": f"Bearer {key}"}, json={"model": model, "temperature": 0, "messages": messages})
                            if resp.status_code != 200:
                                out.append({"id":sid, "label":"neutral"}); continue
                            txt = (resp.json().get('choices',[{}])[0].get('message',{}).get('content','') or '').strip().lower()
                            label = 'contradiction' if 'contradiction' in txt else 'entailment' if 'entail' in txt else 'neutral'
                            out.append({"id": sid, "label": label})
                    return out
                def classify_ollama(pairs):
                    base = os.getenv('PANTHER_OLLAMA_BASE','http://127.0.0.1:11434').rstrip('/')
                    model = os.getenv('PANTHER_OLLAMA_NLI_MODEL','llama3')
                    if not pairs: return []
                    out = []
                    with httpx.Client(timeout=30.0) as client:
                        for sid, premise, hypothesis in pairs:
                            prompt = ("You are an NLI classifier. Reply with one word: entailment, neutral, or contradiction.\n"+
                                      f"Premise: {premise}\nHypothesis: {hypothesis}\nAnswer:")
                            resp = client.post(f"{base}/api/generate", json={"model": model, "prompt": prompt, "stream": False, "options": {"temperature": 0}})
                            if resp.status_code != 200:
                                out.append({"id":sid, "label":"neutral"}); continue
                            txt = (resp.json().get('response','') or '').strip().lower()
                            label = 'contradiction' if 'contradiction' in txt else 'entailment' if 'entail' in txt else 'neutral'
                            out.append({"id": sid, "label": label})
                    return out
                nli_res = classify_openai(pairs) if prov == 'openai' else classify_ollama(pairs)
                if nli_res:
                    n_contra = sum(1 for r in nli_res if r.get('label')=='contradiction')
                    nli_rate = n_contra / max(1, len(nli_res))
                    nli_details = nli_res
            except Exception:
                nli_rate = None
        # Ranking with per-source thresholds
        thresholds = req.source_thresholds or {}
        ranking = []
        for s in scored:
            sid = s.get('id','')
            th = float(thresholds.get(sid, sim_th))
            ranking.append({"id": sid, "score": s.get('score',0.0), "threshold": th, "accepted": (s.get('score',0.0) >= th)})
        justification = " ".join([s['text'] for e in evidences for s in e.get('sentences', [])][:ev_k])
        out = {"score": score, "coverage": coverage, "contradiction_rate": con_rate, "top_sources": topk, "evidence": evidences, "justification": justification, "ranking": ranking}
        if nli_rate is not None:
            out["contradiction_rate_nli"] = nli_rate
            out["nli"] = nli_details
        # Audit insert (if DB available)
        try:
            import time, hashlib
            ts = int(time.time() * 1000)
            norm_text = " ".join(norm(text))
            text_hash = hashlib.sha256(norm_text.encode('utf-8')).hexdigest()
            params = {
                "method": method,
                "top_k": k,
                "similarity_threshold": sim_th,
                "evidence_k": ev_k,
                "contradiction_method": (req.contradiction_method or '').lower(),
                "source_thresholds": thresholds,
            }
            db_insert_factcheck_audit(ts, text_hash, method, (req.contradiction_method or '').lower(), json.dumps(params), json.dumps(out))
        except Exception:
            pass
        return out

    if m in ("bias_adv","bias-adv") and req.samples is not None:
        samples = req.samples
        # Default groups by locale
        if req.groups:
            groups = req.groups
        else:
            loc = (req.locale or "en").lower()
            # base groups by locale
            if loc.startswith("pt"):
                groups = {"masc":["ele","dele"], "fem":["ela","dela"], "neutro":["elu","delu"]}
            else:
                groups = {"male":["he","him","his"], "female":["she","her","hers"], "neutral":["they","them","their"]}
            # domain-specific additions
            dom = (req.domain or "").lower()
            if dom == "healthcare" and not loc.startswith("pt"):
                # include role nouns often gendered
                groups["male"] += ["doctor","surgeon"]
                groups["female"] += ["nurse","caregiver"]
            elif dom == "finance" and not loc.startswith("pt"):
                groups["male"] += ["banker","trader"]
                groups["female"] += ["assistant","clerk"]
        counts: dict[str,int] = {k: 0 for k in groups.keys()}
        per_sample: list[dict] = []
        for idx, s in enumerate(samples):
            low = s.lower()
            present = []
            for g, toks in groups.items():
                c = sum(low.count(t) for t in toks)
                if c > 0:
                    counts[g] = counts.get(g, 0) + c
                    present.append(g)
            per_sample.append({"index": idx, "groups_present": present})
        maxc = max(counts.values()) if counts else 0
        minc = min(counts.values()) if counts else 0
        disparity = 0.0 if maxc == 0 else (maxc - minc) / maxc
        # Simple mitigation suggestions: prefer neutral pronouns when single-group dominance is detected
        suggestions: list[dict] = []
        for idx, s in enumerate(samples):
            ps = per_sample[idx].get('groups_present', [])
            if len(ps) == 1 and (ps[0] == 'male' or ps[0] == 'female' or ps[0] in ('masc','fem')):
                if req.locale and req.locale.lower().startswith('pt'):
                    suggestions.append({"index": idx, "suggestion": "Considere trocar por pronomes neutros (ex.: 'elu/delu') onde for apropriado."})
                else:
                    suggestions.append({"index": idx, "suggestion": "Consider using neutral phrasing (e.g., 'they/them/their') where appropriate."})
        out = {"score": float(disparity), "group_counts": counts, "per_sample": per_sample, "suggestions": suggestions}
        if req.neutralize:
            neutral_texts = []
            for s in samples:
                t = s
                if req.locale and req.locale.lower().startswith('pt'):
                    t = t.replace(' ele ', ' elu ').replace(' ela ', ' elu ').replace(' dele ', ' delu ').replace(' dela ', ' delu ')
                else:
                    t = t.replace(' he ', ' they ').replace(' she ', ' they ').replace(' his ', ' their ').replace(' her ', ' their ')
                neutral_texts.append(t)
            out["neutral_texts"] = neutral_texts
        return out

    # Contextual metrics: domain/locale-driven relevance (must/should keywords)
    if m in ("contextual_relevance", "contextual-relevance") and req.text is not None:
        import re, unicodedata
        def strip_accents(s: str) -> str:
            return ''.join(c for c in unicodedata.normalize('NFD', s) if unicodedata.category(c) != 'Mn')
        def norm(t: str) -> str:
            t = strip_accents(t.lower())
            t = re.sub(r"[^a-z0-9\s]", " ", t)
            return t
        # Defaults per domain/locale
        loc = (req.locale or 'en').lower()
        dom = (req.domain or '').lower()
        defaults = {
            'healthcare': {
                'en': {
                    'must': ['insulin','glucose','hba1c'],
                    'should': ['contraindicated','dosage','adverse','pregnancy','anvisa'],
                },
                'pt': {
                    'must': ['insulina','glicose','hba1c'],
                    'should': ['contraindicado','dosagem','efeitos','gravidez','anvisa'],
                },
            },
            'finance': {
                'en': {
                    'must': ['revenue','profit','risk'],
                    'should': ['roi','forecast','cost'],
                },
                'pt': {
                    'must': ['receita','lucro','risco'],
                    'should': ['roi','projecao','custo'],
                },
            },
        }
        must = req.keywords_must or defaults.get(dom, {}).get(loc, {}).get('must', [])
        should = req.keywords_should or defaults.get(dom, {}).get(loc, {}).get('should', [])
        txt = norm(req.text)
        present_must = [k for k in must if k and norm(k) in txt]
        present_should = [k for k in should if k and norm(k) in txt]
        cov_must = (len(present_must) / max(1, len(must))) if must else 0.0
        cov_should = (len(present_should) / max(1, len(should))) if should else 0.0
        score = max(0.0, min(1.0, 0.7*cov_must + 0.3*cov_should))
        return {
            'score': float(score),
            'domain': dom,
            'locale': loc,
            'must': {'total': len(must), 'present': present_must, 'missing': [k for k in must if k not in present_must]},
            'should': {'total': len(should), 'present': present_should, 'missing': [k for k in should if k not in present_should]},
        }

    # Guided rewrite (advanced mitigation)
    if m in ("bias_rewrite","guided_rewrite","rewrite") and req.text is not None:
        import time, hashlib, httpx  # type: ignore
        # Derive defaults leveraging contextual relevance dictionaries
        loc = (req.locale or 'en').lower()
        dom = (req.domain or '').lower()
        # reuse defaults from contextual relevance
        def_defaults = {
            'healthcare': {
                'en': {'must': ['insulin','glucose','hba1c']},
                'pt': {'must': ['insulina','glicose','hba1c']},
            }
        }
        must = req.keywords_must or def_defaults.get(dom, {}).get(loc, {}).get('must', [])
        text = req.text
        # Simple rule-based neutralization and enrichment
        def neutralize_rule(t: str, locale: str) -> str:
            if locale.startswith('pt'):
                t = (" "+t+" ").replace(' ele ', ' elu ').replace(' ela ', ' elu ').replace(' dele ', ' delu ').replace(' dela ', ' delu ').strip()
            else:
                t = (" "+t+" ").replace(' he ', ' they ').replace(' she ', ' they ').replace(' his ', ' their ').replace(' her ', ' their ').strip()
            return t
        # Append missing must keywords as a footer sentence (rule fallback)
        def enrich_with_keywords(t: str, kws: list[str]) -> str:
            missing = [k for k in kws if k and k.lower() not in t.lower()]
            if missing:
                if loc.startswith('pt'):
                    return t + "\n\n(Sugerido) Incluir termos: " + ", ".join(missing)
                else:
                    return t + "\n\n(Suggested) Include terms: " + ", ".join(missing)
            return t
        method = (req.rewrite_method or 'rule').lower()
        out_text = None
        applied = {"method": method}
        if method == 'llm':
            prov = os.getenv('PANTHER_EMBED_PROVIDER','openai').lower()
            try:
                if prov == 'openai':
                    base = os.getenv('PANTHER_OPENAI_BASE','https://api.openai.com').rstrip('/')
                    key = os.getenv('PANTHER_OPENAI_API_KEY','')
                    model = os.getenv('PANTHER_OPENAI_REWRITE_MODEL','gpt-4o-mini')
                    if not key: raise RuntimeError('missing openai key')
                    style = (req.target_style or 'neutral')
                    locale_hint = "Portuguese (Brazil)" if loc.startswith('pt') else "English"
                    guard = ("Do not unnaturally repeat or stuff keywords; include domain-critical terms only when they fit the context. "
                             "Preserve meaning and correctness.")
                    prompt = (
                        f"You are a writing assistant. Rewrite the text in a {style} style in {locale_hint}, "
                        "using neutral/non-gendered language when applicable. " + guard + "\n" +
                        ("Domain-critical terms to consider: " + ", ".join(must) + "\n" if must else "") +
                        "Text: " + text
                    )
                    with httpx.Client(timeout=30.0) as client:
                        resp = client.post(f"{base}/v1/chat/completions", headers={"Authorization": f"Bearer {key}"}, json={"model": model, "temperature": 0.2, "messages": [{"role":"user","content": prompt}]})
                        resp.raise_for_status()
                        out_text = resp.json().get('choices',[{}])[0].get('message',{}).get('content','')
                        applied["provider"] = 'openai'
                else:
                    base = os.getenv('PANTHER_OLLAMA_BASE','http://127.0.0.1:11434').rstrip('/')
                    model = os.getenv('PANTHER_OLLAMA_REWRITE_MODEL','llama3')
                    style = (req.target_style or 'neutral')
                    locale_hint = "Portuguese (Brazil)" if loc.startswith('pt') else "English"
                    guard = ("Do not unnaturally repeat or stuff keywords; include domain-critical terms only when they fit the context. "
                             "Preserve meaning and correctness.")
                    prompt = (
                        f"Rewrite the text in a {style} style in {locale_hint}, using neutral/non-gendered language when applicable. " + guard + "\n" +
                        ("Domain-critical terms to consider: " + ", ".join(must) + "\n" if must else "") +
                        "Text: " + text
                    )
                    with httpx.Client(timeout=30.0) as client:
                        resp = client.post(f"{base}/api/generate", json={"model": model, "prompt": prompt, "stream": False, "options": {"temperature": 0.2}})
                        resp.raise_for_status()
                        out_text = resp.json().get('response','')
                        applied["provider"] = 'ollama'
            except Exception as e:
                out_text = None
                applied["error"] = str(e)
        if not out_text:
            # fallback to rule-based
            t = neutralize_rule(text, loc)
            t = enrich_with_keywords(t, must)
            if (req.target_style or '').lower() == 'formal':
                t = ("Please note: " + t[0].upper() + t[1:] if t else t)
            out_text = t
            applied["provider"] = 'rule'
        # Audit save
        try:
            ts = int(time.time()*1000)
            text_hash = hashlib.sha256(text.encode('utf-8')).hexdigest()
            db_insert_rewrite_audit(ts, text_hash, json.dumps({"domain":dom, "locale":loc, "method":method, "style": req.target_style or 'neutral', "must": must}), out_text)
        except Exception:
            pass
        return {"rewritten": out_text, "applied": applied}
    return {"error": "unsupported or missing fields"}


@router.get("/metrics")
def metrics_exporter():
    if not _PROM:
        raise HTTPException(status_code=503, detail="prometheus not enabled")
    return Response(content=generate_latest(), media_type=CONTENT_TYPE_LATEST)  # type: ignore
