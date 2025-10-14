import {NativeModules} from 'react-native';

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
