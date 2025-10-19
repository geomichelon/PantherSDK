import {NativeModules, Platform} from 'react-native';

type PantherModuleType = {
  init(): Promise<number>;
  generate(prompt: string): Promise<string>;
  metricsBleu(reference: string, candidate: string): Promise<number>;
  recordMetric(name: string): Promise<number>;
  listStorageItems(): Promise<string>;
  getLogs(): Promise<string>;
  validate(prompt: string): Promise<string>;
  validateMulti(prompt: string, providersJson: string): Promise<string>;
  version(): Promise<string>;
  validateMultiWithProof(prompt: string, providersJson: string): Promise<string>;
  validateCustomWithProof(prompt: string, providersJson: string, guidelinesJson: string): Promise<string>;
  validateOpenAI(prompt: string, apiKey: string, model: string, base: string): Promise<string>;
  validateOllama(prompt: string, base: string, model: string): Promise<string>;
  validateCustom(prompt: string, providersJson: string, guidelinesJson: string): Promise<string>;
  tokenCount(text: string): Promise<number>;
  calculateCost(tokensIn: number, tokensOut: number, providerName: string, costRulesJson: string): Promise<number>;
};

const {PantherModule} = NativeModules as {PantherModule: PantherModuleType};

export async function init(): Promise<void> {
  await PantherModule.init();
}

export async function generate(prompt: string): Promise<string> {
  return PantherModule.generate(prompt);
}

export async function metricsBleu(reference: string, candidate: string): Promise<number> {
  return PantherModule.metricsBleu(reference, candidate);
}

export async function recordMetric(name: string): Promise<number> {
  return PantherModule.recordMetric(name);
}

export async function listStorageItems(): Promise<string> {
  return PantherModule.listStorageItems();
}

export async function getLogs(): Promise<string> {
  return PantherModule.getLogs();
}

export async function validate(prompt: string): Promise<string> {
  return PantherModule.validate(prompt);
}

export async function validateMulti(prompt: string, providersJson: string): Promise<string> {
  return PantherModule.validateMulti(prompt, providersJson);
}

export async function version(): Promise<string> {
  return PantherModule.version();
}

export async function validateMultiWithProof(prompt: string, providersJson: string): Promise<string> {
  return PantherModule.validateMultiWithProof(prompt, providersJson);
}

export async function validateCustomWithProof(prompt: string, providersJson: string, guidelinesJson: string): Promise<string> {
  return PantherModule.validateCustomWithProof(prompt, providersJson, guidelinesJson);
}

export async function validateOpenAI(prompt: string, apiKey: string, model: string, base: string): Promise<string> {
  return PantherModule.validateOpenAI(prompt, apiKey, model, base);
}

export async function validateOllama(prompt: string, base: string, model: string): Promise<string> {
  return PantherModule.validateOllama(prompt, base, model);
}

export async function validateCustom(prompt: string, providersJson: string, guidelinesJson: string): Promise<string> {
  return PantherModule.validateCustom(prompt, providersJson, guidelinesJson);
}

export async function tokenCount(text: string): Promise<number> {
  return PantherModule.tokenCount(text);
}

export async function calculateCost(tokensIn: number, tokensOut: number, providerName: string, costRulesJson: string): Promise<number> {
  return PantherModule.calculateCost(tokensIn, tokensOut, providerName, costRulesJson);
}

export async function anchorProof(hash: string, apiBase?: string, apiKey?: string): Promise<{tx_hash?: string; error?: string}> {
  const base = apiBase ?? (Platform.OS === 'android' ? 'http://10.0.2.2:8000' : 'http://127.0.0.1:8000');
  const res = await fetch(`${base}/proof/anchor`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      ...(apiKey ? {'X-API-Key': apiKey} : {}),
    },
    body: JSON.stringify({hash: `0x${hash}`}),
  });
  return res.json();
}

// --- Agents (Stage 6) ---
export async function runAgent(plan: any, input: any, apiBase?: string, apiKey?: string, asyncRun: boolean = true): Promise<{run_id?: string; result?: any; status?: string; error?: string}> {
  const base = apiBase ?? (Platform.OS === 'android' ? 'http://10.0.2.2:8000' : 'http://127.0.0.1:8000');
  const res = await fetch(`${base}/agent/run`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      ...(apiKey ? {'X-API-Key': apiKey} : {}),
    },
    body: JSON.stringify({ plan, input, async_run: asyncRun }),
  });
  return res.json();
}

export async function getAgentStatus(runId: string, apiBase?: string, apiKey?: string): Promise<{run_id: string; status: string; done: boolean; error?: string}> {
  const base = apiBase ?? (Platform.OS === 'android' ? 'http://10.0.2.2:8000' : 'http://127.0.0.1:8000');
  const res = await fetch(`${base}/agent/status?run_id=${encodeURIComponent(runId)}`, {
    headers: {
      ...(apiKey ? {'X-API-Key': apiKey} : {}),
    },
  });
  return res.json();
}

export async function getAgentEvents(runId: string, apiBase?: string, apiKey?: string): Promise<{run_id: string; events: any[]; error?: string}> {
  const base = apiBase ?? (Platform.OS === 'android' ? 'http://10.0.2.2:8000' : 'http://127.0.0.1:8000');
  const res = await fetch(`${base}/agent/events?run_id=${encodeURIComponent(runId)}`, {
    headers: {
      ...(apiKey ? {'X-API-Key': apiKey} : {}),
    },
  });
  return res.json();
}

export async function startAgent(plan: any, input: any, apiBase?: string, apiKey?: string): Promise<{run_id?: string; error?: string}> {
  const base = apiBase ?? (Platform.OS === 'android' ? 'http://10.0.2.2:8000' : 'http://127.0.0.1:8000');
  const res = await fetch(`${base}/agent/start`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      ...(apiKey ? {'X-API-Key': apiKey} : {}),
    },
    body: JSON.stringify({ plan, input })
  });
  return res.json();
}

export async function pollAgent(runId: string, cursor: number, apiBase?: string, apiKey?: string): Promise<{events: any[]; cursor: number; done: boolean; error?: string}> {
  const base = apiBase ?? (Platform.OS === 'android' ? 'http://10.0.2.2:8000' : 'http://127.0.0.1:8000');
  const res = await fetch(`${base}/agent/poll?run_id=${encodeURIComponent(runId)}&cursor=${cursor}`, {
    headers: {
      ...(apiKey ? {'X-API-Key': apiKey} : {}),
    },
  });
  return res.json();
}

// --- Metrics helpers ---
export async function evaluatePlagiarism(text: string, samples: string[], apiBase?: string, apiKey?: string, ngram?: number): Promise<{score?: number; error?: string}> {
  const base = apiBase ?? (Platform.OS === 'android' ? 'http://10.0.2.2:8000' : 'http://127.0.0.1:8000');
  const res = await fetch(`${base}/metrics/evaluate`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      ...(apiKey ? {'X-API-Key': apiKey} : {}),
    },
    body: JSON.stringify({ metric: 'plagiarism', text, samples, ...(ngram ? {ngram} : {}) }),
  });
  return res.json();
}

export async function analyzeBias(samples: string[], apiBase?: string, apiKey?: string): Promise<{bias_score?: number; group_counts?: any; error?: string}> {
  const base = apiBase ?? (Platform.OS === 'android' ? 'http://10.0.2.2:8000' : 'http://127.0.0.1:8000');
  const res = await fetch(`${base}/bias/analyze`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      ...(apiKey ? {'X-API-Key': apiKey} : {}),
    },
    body: JSON.stringify({ samples }),
  });
  return res.json();
}
