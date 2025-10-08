import {NativeModules} from 'react-native';

type PantherModuleType = {
  init(): Promise<number>;
  generate(prompt: string): Promise<string>;
  metricsBleu(reference: string, candidate: string): Promise<number>;
  recordMetric(name: string): Promise<number>;
  listStorageItems(): Promise<string>;
  getLogs(): Promise<string>;
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
