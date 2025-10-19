// @ts-nocheck
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
import {init, validate, validateMulti, validateMultiWithProof, validateCustom, validateCustomWithProof, validateOpenAI, validateOllama, anchorProof, startAgent, pollAgent, evaluatePlagiarism, analyzeBias, tokenCount, calculateCost} from './Panther';
// Optional persistence for cost rules
let AsyncStorage: any = null;
try { AsyncStorage = require('@react-native-async-storage/async-storage').default || require('@react-native-async-storage/async-storage'); } catch (_) { AsyncStorage = null; }

const defaultCostRules = `[
  {"match": "openai:gpt-4o-mini",  "usd_per_1k_in": 0.15, "usd_per_1k_out": 0.60},
  {"match": "openai:gpt-4.1-mini", "usd_per_1k_in": 0.30, "usd_per_1k_out": 1.20},
  {"match": "openai:gpt-4.1",      "usd_per_1k_in": 5.00,  "usd_per_1k_out": 15.00},
  {"match": "openai:gpt-4o",       "usd_per_1k_in": 5.00,  "usd_per_1k_out": 15.00},
  {"match": "openai:chatgpt-5",    "usd_per_1k_in": 5.00,  "usd_per_1k_out": 15.00},
  {"match": "ollama:llama3",       "usd_per_1k_in": 0.00,  "usd_per_1k_out": 0.00},
  {"match": "ollama:phi3",         "usd_per_1k_in": 0.00,  "usd_per_1k_out": 0.00},
  {"match": "ollama:mistral",      "usd_per_1k_in": 0.00,  "usd_per_1k_out": 0.00}
]`;
const openAIModels = ['gpt-4o-mini', 'gpt-4.1-mini', 'gpt-4.1', 'gpt-4o', 'chatgpt-5'];
const ollamaModels = ['llama3', 'phi3', 'mistral'];
const anthropicModels = ['claude-3-5-sonnet-latest', 'claude-3-opus-latest', 'claude-3-haiku-latest'];

export default function AppSample() {
  type Mode = 'single' | 'multi' | 'proof';
  type Provider = 'openai' | 'ollama' | 'anthropic' | 'default';
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
  const [mode, setMode] = useState<Mode>('proof');
  const [provider, setProvider] = useState<Provider>('openai');
  const [openAIBase, setOpenAIBase] = useState<string>('https://api.openai.com');
  const [openAIModel, setOpenAIModel] = useState<string>('gpt-4o-mini');
  const [apiKeyOpenAI, setApiKeyOpenAI] = useState<string>('');
  const [ollamaBase, setOllamaBase] = useState<string>('http://127.0.0.1:11434');
  const [ollamaModel, setOllamaModel] = useState<string>('llama3');
  const [anthropicBase, setAnthropicBase] = useState<string>('https://api.anthropic.com');
  const [anthropicModel, setAnthropicModel] = useState<string>('claude-3-5-sonnet-latest');
  const [apiKeyAnthropic, setApiKeyAnthropic] = useState<string>('');
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
  const [biasText, setBiasText] = useState('');
  const [biasScore, setBiasScore] = useState<number | null>(null);
  const [trustIndex, setTrustIndex] = useState<number | null>(null);
  const [costRules, setCostRules] = useState<string>(defaultCostRules);
  const [showCostEditor, setShowCostEditor] = useState<boolean>(false);
  const [useCustomGuidelines, setUseCustomGuidelines] = useState<boolean>(false);
  const [guidelinesJson, setGuidelinesJson] = useState<string>('');
  const [guidelinesURL, setGuidelinesURL] = useState<string>('');
  const [lastResults, setLastResults] = useState<any[]>([]);

  useEffect(() => {
    init().catch(() => undefined);
    // Load saved cost rules if available
    (async () => {
      try {
        if (AsyncStorage) {
          const s = await AsyncStorage.getItem('panther.cost_rules');
          if (s && s.trim().length) setCostRules(s);
        }
      } catch (_) {}
    })();
  }, []);

  const runValidation = async () => {
    setTxHash(null);
    try {
      const doCustom = useCustomGuidelines && guidelinesJson.trim().length > 0;
      let raw: string;
      if (mode === 'single') {
        if (provider === 'openai') {
          raw = await validateOpenAI(prompt, apiKeyOpenAI, openAIModel, openAIBase);
        } else if (provider === 'ollama') {
          raw = await validateOllama(prompt, ollamaBase, ollamaModel);
        } else if (provider === 'anthropic') {
          const singleAnthropic = JSON.stringify([{ type: 'anthropic', base_url: anthropicBase, model: anthropicModel, api_key: apiKeyAnthropic }]);
          raw = await validateMulti(prompt, singleAnthropic);
        } else {
          raw = await validate(prompt);
        }
      } else if (mode === 'multi') {
        raw = doCustom ? await validateCustom(prompt, providersJson, guidelinesJson)
                       : await validateMulti(prompt, providersJson);
      } else {
        raw = doCustom ? await validateCustomWithProof(prompt, providersJson, guidelinesJson)
                       : await validateMultiWithProof(prompt, providersJson);
      }
      const data = JSON.parse(raw);
      if (Array.isArray(data)) {
        // Fallback from validateMulti
        setLines(data.map((r: any) => `${r.provider_name ?? '?'} – ${(r.adherence_score ?? 0).toFixed(1)}% – ${r.latency_ms ?? 0} ms`));
        setProof(null);
        return;
      }
      const res = (data.results as any[]) || [];
      setLastResults(res);
      const tin = await tokenCount(prompt);
      const lns: string[] = [];
      for (const r of res) {
        const name = r.provider_name ?? '?';
        const score = Number(r.adherence_score ?? 0);
        const lat = Number(r.latency_ms ?? 0);
        const text = String(r.raw_text ?? '');
        const tout = await tokenCount(text);
        const rules = costRules && costRules.trim().length ? costRules : defaultCostRules;
        const cost = await calculateCost(tin, tout, name, rules);
        lns.push(`${name} – ${score.toFixed(1)}% – ${lat} ms – ${tin}/${tout} tok – $${cost.toFixed(4)}`);
      }
      setLines(lns);
      setProof(mode === 'proof' ? (data.proof?.combined_hash || null) : null);
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
      if (!resp.tx_hash && resp.error) setLines((prev: string[]) => [...prev, `Anchor error: ${resp.error}`]);
    } catch (e: any) {
      setLines((prev: string[]) => [...prev, `Anchor error: ${String(e?.message || e)}`]);
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
        .map((s: string) => s.trim())
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

  const runBias = async () => {
    try {
      const samples = biasText
        .split('\n')
        .map((s: string) => s.trim())
        .filter(s => s.length > 0);
      const res = await analyzeBias(samples, apiBase, apiKey || undefined);
      if ((res as any).bias_score !== undefined) setBiasScore(Number((res as any).bias_score));
      // Compute trust index (avg adherence penalized by bias)
      try {
        const arr = lastResults || [];
        const vals = arr.map((r: any) => Number(r.adherence_score || 0) / 100.0).filter((v: number) => !isNaN(v));
        const avg = vals.length ? (vals.reduce((a: number,b: number)=>a+b,0) / vals.length) : 0;
        const bias = (res as any).bias_score ? Number((res as any).bias_score) : 0;
        setTrustIndex(Math.max(0, Math.min(1, avg * (1 - bias))));
      } catch {}
    } catch {
      setBiasScore(null);
    }
  };

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <Text style={styles.h1}>Panther SDK – React Native Sample</Text>

      <Text style={styles.h2}>Prompt</Text>
      <TextInput style={styles.input} value={prompt} onChangeText={setPrompt} />

      <Text style={styles.h2}>Execução</Text>
      <View style={styles.row}>
        <Button title={mode === 'single' ? 'Single' : 'single'} onPress={() => setMode('single')} />
        <View style={{width:8}} />
        <Button title={mode === 'multi' ? 'Multi' : 'multi'} onPress={() => setMode('multi')} />
        <View style={{width:8}} />
        <Button title={mode === 'proof' ? 'With Proof' : 'with proof'} onPress={() => setMode('proof')} />
      </View>

      <View style={{height:8}} />
      <View style={styles.row}>
        <Button title={provider === 'openai' ? 'OpenAI' : 'openai'} onPress={() => setProvider('openai')} />
        <View style={{width:8}} />
        <Button title={provider === 'ollama' ? 'Ollama' : 'ollama'} onPress={() => setProvider('ollama')} />
        <View style={{width:8}} />
        <Button title={provider === 'anthropic' ? 'Anthropic' : 'anthropic'} onPress={() => setProvider('anthropic')} />
        <View style={{width:8}} />
        <Button title={provider === 'default' ? 'Default' : 'default'} onPress={() => setProvider('default')} />
      </View>

      {provider === 'openai' ? (
        <>
          <TextInput style={styles.input} value={apiKeyOpenAI} onChangeText={setApiKeyOpenAI} placeholder="OpenAI API Key" secureTextEntry />
          <TextInput style={styles.input} value={openAIBase} onChangeText={setOpenAIBase} placeholder="Base URL" />
          <TextInput style={styles.input} value={openAIModel} onChangeText={setOpenAIModel} placeholder="Model (e.g., gpt-4o-mini)" />
          <View style={[styles.row, {flexWrap: 'wrap'}]}>
            {openAIModels.map((m) => (
              <Button key={m} title={m} onPress={() => setOpenAIModel(m)} />
            ))}
          </View>
        </>
      ) : provider === 'ollama' ? (
        <>
          <TextInput style={styles.input} value={ollamaBase} onChangeText={setOllamaBase} placeholder="Ollama Base (http://127.0.0.1:11434)" />
          <TextInput style={styles.input} value={ollamaModel} onChangeText={setOllamaModel} placeholder="Ollama Model (e.g., llama3)" />
          <View style={[styles.row, {flexWrap: 'wrap'}]}>
            {ollamaModels.map((m) => (
              <Button key={m} title={m} onPress={() => setOllamaModel(m)} />
            ))}
          </View>
        </>
      ) : provider === 'anthropic' ? (
        <>
          <TextInput style={styles.input} value={apiKeyAnthropic} onChangeText={setApiKeyAnthropic} placeholder="Anthropic API Key" secureTextEntry />
          <TextInput style={styles.input} value={anthropicBase} onChangeText={setAnthropicBase} placeholder="Base URL (https://api.anthropic.com)" />
          <TextInput style={styles.input} value={anthropicModel} onChangeText={setAnthropicModel} placeholder="Model (e.g., claude-3-5-sonnet-latest)" />
          <View style={[styles.row, {flexWrap: 'wrap'}]}>
            {anthropicModels.map((m) => (
              <Button key={m} title={m} onPress={() => setAnthropicModel(m)} />
            ))}
          </View>
        </>
      ) : (
        <Text style={{color:'#666'}}>Usando providers de ambiente (Default)</Text>
      )}

      <Text style={styles.h2}>Providers JSON</Text>
      <TextInput
        style={[styles.input, styles.multiline]}
        value={providersJson}
        onChangeText={setProvidersJson}
        multiline
      />

      <Text style={styles.h2}>Diretrizes</Text>
      <View style={styles.row}>
        <Text>Usar diretrizes customizadas (JSON)</Text>
        <Switch value={useCustomGuidelines} onValueChange={setUseCustomGuidelines} style={{marginLeft: 8}} />
      </View>
      {useCustomGuidelines ? (
        <>
          <TextInput
            style={[styles.input, styles.multiline]}
            value={guidelinesJson}
            onChangeText={setGuidelinesJson}
            multiline
            placeholder='[ { "topic": "...", "expected_terms": ["..."] } ]'
          />
          <View style={styles.row}>
            <TextInput
              style={[styles.input, {flex: 1}]}
              value={guidelinesURL}
              onChangeText={setGuidelinesURL}
              placeholder="https://example.com/guidelines.json"
            />
            <View style={{width:8}} />
            <Button title="Carregar" onPress={async () => {
              try { const r = await fetch(guidelinesURL); const s = await r.text(); setGuidelinesJson(s); } catch {}
            }} />
          </View>
        </>
      ) : (
        <Text style={{color:'#666'}}>ANVISA (padrão embutido)</Text>
      )}

      <Text style={styles.h2}>Backend API</Text>
      <TextInput style={styles.input} value={apiBase} onChangeText={setApiBase} placeholder="http://127.0.0.1:8000" />
      <TextInput
        style={styles.input}
        value={apiKey}
        onChangeText={setApiKey}
        placeholder="API Key (X-API-Key) - optional"
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

      <Text style={styles.h2}>Cost Rules (JSON)</Text>
      <View style={styles.row}>
        <Button title={showCostEditor ? 'Hide' : 'Edit'} onPress={() => setShowCostEditor(v => !v)} />
        <View style={{width: 8}} />
        <Button title="Restore Default" onPress={() => setCostRules(defaultCostRules)} />
        <View style={{width: 8}} />
        <Button
          title="Save"
          onPress={async () => {
            try { if (AsyncStorage) await AsyncStorage.setItem('panther.cost_rules', costRules); } catch (_) {}
          }}
        />
      </View>
      {showCostEditor ? (
        <TextInput
          style={[styles.input, styles.multiline]}
          value={costRules}
          onChangeText={setCostRules}
          multiline
        />
      ) : null}

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

      <Text style={styles.h2}>Compliance (Bias)</Text>
      <Text style={{marginBottom: 6}}>Samples (one per line):</Text>
      <TextInput
        style={[styles.input, styles.multiline]}
        value={biasText}
        onChangeText={setBiasText}
        multiline
      />
      <View style={styles.row}>
        <Button title="Analyze Bias" onPress={runBias} />
        {biasScore !== null ? <Text style={{marginLeft: 12}}>bias_score: {biasScore?.toFixed(3)}</Text> : null}
        {trustIndex !== null ? <Text style={{marginLeft: 12}}>trust_index: {(trustIndex*100).toFixed(1)}%</Text> : null}
      </View>

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
