import React, {useEffect, useState} from 'react';
import {View, Text, TextInput, Button, ScrollView, Platform, StyleSheet, Linking, Switch} from 'react-native';
// Try to load EventSource implementation for RN (optional)
let RNEventSource: any = null;
try {
  // eslint-disable-next-line @typescript-eslint/no-var-requires
  RNEventSource = require('react-native-event-source').default || require('react-native-event-source');
} catch (_) {
  RNEventSource = null;
}
import {init, validateMultiWithProof, anchorProof, startAgent, pollAgent, evaluatePlagiarism} from './Panther';

export default function AppSample() {
  const [prompt, setPrompt] = useState('Explain insulin function');
  const [providersJson, setProvidersJson] = useState(
    JSON.stringify(
      [
        {
          type: 'openai',
          base_url: 'https://api.openai.com',
          model: 'gpt-4o-mini',
          api_key: '',
        },
      ],
      null,
      2,
    ),
  );
  const [apiBase, setApiBase] = useState(
    Platform.OS === 'android' ? 'http://10.0.2.2:8000' : 'http://127.0.0.1:8000',
  );
  const [apiKey, setApiKey] = useState('');
  const [lines, setLines] = useState<string[]>([]);
  const [proof, setProof] = useState<string | null>(null);
  const [txHash, setTxHash] = useState<string | null>(null);
  const [statusText, setStatusText] = useState<string | null>(null);
  const [explorerUrl, setExplorerUrl] = useState<string | null>(null);
  const [contractUrl, setContractUrl] = useState<string | null>(null);
  const [runId, setRunId] = useState<string | null>(null);
  const [agentStatus, setAgentStatus] = useState<string | null>(null);
  const [agentEvents, setAgentEvents] = useState<string[]>([]);
  const [useSSE, setUseSSE] = useState<boolean>(false);
  const [plagCorpus, setPlagCorpus] = useState('Insulin regulates glucose in the blood.\nVitamin C supports the immune system.');
  const [plagNgram, setPlagNgram] = useState<string>('3');
  const [plagScore, setPlagScore] = useState<number | null>(null);

  useEffect(() => {
    init().catch(() => undefined);
  }, []);

  const runValidation = async () => {
    setTxHash(null);
    try {
      const raw = await validateMultiWithProof(prompt, providersJson);
      const data = JSON.parse(raw);
      if (Array.isArray(data)) {
        // Fallback from validateMulti
        setLines(data.map((r: any) => `${r.provider_name ?? '?'} – ${(r.adherence_score ?? 0).toFixed(1)}% – ${r.latency_ms ?? 0} ms`));
        setProof(null);
        return;
      }
      const res = data.results as any[];
      const lns = (res || []).map(
        (r) => `${r.provider_name ?? '?'} – ${(r.adherence_score ?? 0).toFixed(1)}% – ${r.latency_ms ?? 0} ms`,
      );
      setLines(lns);
      setProof(data.proof?.combined_hash || null);
    } catch (e: any) {
      setLines([String(e?.message || e)]);
      setProof(null);
    }
  };

  const doAnchor = async () => {
    if (!proof) return;
    try {
      const resp = await anchorProof(proof, apiBase, apiKey || undefined);
      setTxHash(resp.tx_hash || null);
      setExplorerUrl((resp as any).explorer_url || null);
      if (!resp.tx_hash && resp.error) setLines((prev) => [...prev, `Anchor error: ${resp.error}`]);
    } catch (e: any) {
      setLines((prev) => [...prev, `Anchor error: ${String(e?.message || e)}`]);
    }
  };

  const checkStatus = async () => {
    if (!proof) return;
    try {
      const base = apiBase || (Platform.OS === 'android' ? 'http://10.0.2.2:8000' : 'http://127.0.0.1:8000');
      const res = await fetch(`${base}/proof/status?hash=0x${proof}`, {
        headers: apiKey ? {'X-API-Key': apiKey} : undefined,
      });
      const data = await res.json();
      setStatusText(`Anchored: ${data.anchored ? 'true' : 'false'}`);
      setContractUrl(data.contract_url || null);
    } catch (e: any) {
      setStatusText(`Status error: ${String(e?.message || e)}`);
    }
  };

  const runAgentFlow = async () => {
    setLines([]);
    setAgentEvents([]);
    setRunId(null);
    setAgentStatus('starting');
    try {
      let providers: any[] = [];
      try { providers = JSON.parse(providersJson); } catch { providers = []; }
      const plan = { type: 'ValidateSealAnchor' } as any;
      const input = { prompt, providers } as any;
      const res = await startAgent(plan, input, apiBase, apiKey || undefined);
      if ((res as any).error || !(res as any).run_id) {
        setAgentStatus(`error: ${(res as any).error}`);
        return;
      }
      const id = (res as any).run_id as string;
      setRunId(id);
      const base = apiBase || (Platform.OS === 'android' ? 'http://10.0.2.2:8000' : 'http://127.0.0.1:8000');
      // Try SSE when toggled and no API key is required (EventSource doesn't support custom headers)
      const ES: any = RNEventSource || (global as any).EventSource;
      if (useSSE && ES && !apiKey) {
        try {
          const events: any[] = [];
          const es = new ES(`${base}/agent/events/stream?run_id=${encodeURIComponent(id)}`);
          es.onmessage = (msg: any) => {
            try {
              const ev = JSON.parse(msg.data);
              events.push(ev);
              const evLines = events.map((e: any) => `${new Date(e.ts||0).toLocaleTimeString()} [${e.stage}] ${e.message}`);
              setAgentEvents(evLines);
              setAgentStatus('running');
            } catch {}
          };
          es.onerror = () => { try { es.close(); } catch {} setAgentStatus('done'); };
          return;
        } catch {
          // fallback to polling
        }
      }
      // incremental polling
      let cursor = 0;
      let done = false;
      const events: any[] = [];
      for (let i = 0; i < 600; i++) {
        const pol = await pollAgent(id, cursor, apiBase, apiKey || undefined);
        if ((pol as any).error) break;
        const newEvs = pol.events || [];
        for (const ev of newEvs) {
          events.push(ev);
        }
        cursor = pol.cursor || cursor;
        done = !!pol.done;
        setAgentStatus(done ? 'done' : 'running');
        // Update UI incrementally
        const evLines = events.map((e: any) => `${new Date(e.ts||0).toLocaleTimeString()} [${e.stage}] ${e.message}`);
        setAgentEvents(evLines);
        if (done) break;
        await new Promise(r => setTimeout(r, 200));
      }
      const lines: string[] = [];
      // Extract validation summary
      const v = events.find((e: any) => e.stage === 'validate' && (e.message || '').includes('complete'));
      if (v && Array.isArray(v.data)) {
        for (const r of v.data) {
          lines.push(`${r.provider_name ?? '?'} – ${(r.adherence_score ?? 0).toFixed(1)}% – ${r.latency_ms ?? 0} ms`);
        }
      }
      setLines(lines.length ? lines : ['(no results)']);
      // Extract proof hash
      const s = events.find((e: any) => e.stage === 'seal');
      const hash = s && s.data && (s.data.combined_hash || (s.data.proof && s.data.proof.combined_hash));
      if (hash) setProof(hash);
    } catch (e: any) {
      setAgentStatus(`error: ${String(e?.message || e)}`);
    }
  };

  const runPlagiarism = async () => {
    try {
      const samples = plagCorpus
        .split('\n')
        .map(s => s.trim())
        .filter(s => s.length > 0);
      const cand = lines[0] && lines[0].includes('–') ? lines[0] : 'Insulin regulates glucose in the blood.';
      const n = parseInt(plagNgram, 10);
      const res = await evaluatePlagiarism(cand, samples, apiBase, apiKey || undefined, isNaN(n) ? undefined : n);
      if ((res as any).score !== undefined) setPlagScore(Number((res as any).score));
      else setPlagScore(null);
    } catch {
      setPlagScore(null);
    }
  };

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <Text style={styles.h1}>Panther SDK – React Native Sample</Text>

      <Text style={styles.h2}>Prompt</Text>
      <TextInput style={styles.input} value={prompt} onChangeText={setPrompt} />

      <Text style={styles.h2}>Providers JSON</Text>
      <TextInput
        style={[styles.input, styles.multiline]}
        value={providersJson}
        onChangeText={setProvidersJson}
        multiline
      />

      <Text style={styles.h2}>Backend API</Text>
      <TextInput style={styles.input} value={apiBase} onChangeText={setApiBase} placeholder="http://127.0.0.1:8000" />
      <TextInput
        style={styles.input}
        value={apiKey}
        onChangeText={setApiKey}
        placeholder="API Key (X-API-Key) — optional"
        secureTextEntry
      />

      <View style={styles.row}>
        <Button title="Validate" onPress={runValidation} />
        {proof ? <View style={{width: 12}} /> : null}
        {proof ? <Button title="Anchor Proof" onPress={doAnchor} /> : null}
        {proof ? <View style={{width: 12}} /> : null}
        {proof ? <Button title="Check Status" onPress={checkStatus} /> : null}
      </View>

      <View style={styles.row}>
        <Button title="Run Agent" onPress={runAgentFlow} />
        {runId ? <Text style={{marginLeft: 12}}>Run: {runId}</Text> : null}
      </View>
      <View style={[styles.row, {marginTop: 8}]}> 
        <Text>Use SSE</Text>
        <Switch value={useSSE} onValueChange={setUseSSE} style={{marginLeft: 8}} />
      </View>

      {proof ? <Text style={styles.proof}>Proof: {proof}</Text> : null}
      {txHash ? <Text style={styles.proof}>Anchored tx: {txHash}</Text> : null}
      {explorerUrl ? (
        <Button title="View on Explorer" onPress={() => Linking.openURL(explorerUrl!)} />
      ) : null}
      {statusText ? <Text style={styles.proof}>{statusText}</Text> : null}
      {contractUrl ? (
        <Button title="View Contract" onPress={() => Linking.openURL(contractUrl!)} />
      ) : null}

      {agentStatus ? <Text style={styles.proof}>Agent status: {agentStatus}</Text> : null}

      <Text style={styles.h2}>Plagiarism (Jaccard n-gram)</Text>
      <Text style={{marginBottom: 6}}>Corpus (one per line):</Text>
      <TextInput
        style={[styles.input, styles.multiline]}
        value={plagCorpus}
        onChangeText={setPlagCorpus}
        multiline
      />
      <View style={styles.row}>
        <Button title="Check Plagiarism" onPress={runPlagiarism} />
        {plagScore !== null ? (
          <Text style={{marginLeft: 12}}>Score: {plagScore?.toFixed(2)}</Text>
        ) : null}
        <Text style={{marginLeft: 12}}>n-gram:</Text>
        <TextInput
          style={[styles.input, {width: 50, marginLeft: 6}]}
          keyboardType="number-pad"
          value={plagNgram}
          onChangeText={setPlagNgram}
        />
      </View>

      {!!agentEvents.length && (
        <>
          <Text style={styles.h2}>Agent Events</Text>
          {agentEvents.map((l, i) => (
            <Text key={`ev-${i}`} style={styles.line}>{l}</Text>
          ))}
        </>
      )}

      <Text style={styles.h2}>Results</Text>
      {lines.map((l, i) => (
        <Text key={i} style={styles.line}>
          {l}
        </Text>
      ))}
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {padding: 16},
  h1: {fontSize: 20, fontWeight: '600', marginBottom: 12},
  h2: {fontSize: 16, fontWeight: '500', marginTop: 16, marginBottom: 6},
  input: {
    borderWidth: 1,
    borderColor: '#ccc',
    borderRadius: 6,
    padding: 8,
  },
  multiline: {minHeight: 120, textAlignVertical: 'top' as any},
  row: {flexDirection: 'row', alignItems: 'center', marginTop: 12},
  line: {marginVertical: 2},
  proof: {fontSize: 12, color: '#666', marginTop: 8},
});
