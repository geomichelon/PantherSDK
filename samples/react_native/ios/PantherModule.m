#import <React/RCTBridgeModule.h>
#include "panther.h"

@interface RCT_EXTERN_MODULE(PantherModule, NSObject)
- (void)init:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
- (void)generate:(NSString *)prompt resolver:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
- (void)metricsBleu:(NSString *)reference candidate:(NSString *)candidate resolver:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
 - (void)recordMetric:(NSString *)name resolver:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
 - (void)listStorageItems:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
 - (void)getLogs:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
 - (void)validate:(NSString *)prompt resolver:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
 - (void)validateMultiWithProof:(NSString *)prompt providersJson:(NSString *)providersJson resolver:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
 - (void)validateOpenAI:(NSString *)prompt apiKey:(NSString *)apiKey model:(NSString *)model base:(NSString *)base resolver:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
 - (void)validateOllama:(NSString *)prompt base:(NSString *)base model:(NSString *)model resolver:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
 - (void)validateMulti:(NSString *)prompt providersJson:(NSString *)providersJson resolver:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
 - (void)validateCustom:(NSString *)prompt providersJson:(NSString *)providersJson guidelinesJson:(NSString *)guidelinesJson resolver:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
 - (void)validateCustomWithProof:(NSString *)prompt providersJson:(NSString *)providersJson guidelinesJson:(NSString *)guidelinesJson resolver:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
 - (void)version:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
 - (void)tokenCount:(NSString *)text resolver:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
 - (void)calculateCost:(nonnull NSNumber *)tokensIn tokensOut:(nonnull NSNumber *)tokensOut providerName:(NSString *)providerName costRulesJson:(NSString *)costRulesJson resolver:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
<<<<<<< HEAD
 // Guidelines
 - (void)guidelinesIngest:(NSString *)json resolver:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
 - (void)guidelinesScores:(NSString *)query topK:(nonnull NSNumber *)topK method:(NSString *)method resolver:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
 - (void)guidelinesSave:(NSString *)name json:(NSString *)json resolver:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
 - (void)guidelinesLoad:(NSString *)name resolver:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
 - (void)guidelinesBuildEmbeddings:(NSString *)method resolver:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
=======
>>>>>>> origin/main
@end

@implementation PantherModule

RCT_EXPORT_MODULE();

RCT_REMAP_METHOD(init,
                 initWithResolver:(RCTPromiseResolveBlock)resolve
                 rejecter:(RCTPromiseRejectBlock)reject)
{
  int rc = panther_init();
  if (rc == 0) { resolve(@(rc)); }
  else { reject(@"ERR_INIT", @"panther_init failed", nil); }
}

RCT_REMAP_METHOD(generate,
                 generateWithPrompt:(NSString *)prompt
                 resolver:(RCTPromiseResolveBlock)resolve
                 rejecter:(RCTPromiseRejectBlock)reject)
{
  const char* cPrompt = [prompt UTF8String];
  char* out = panther_generate(cPrompt);
  NSString* res = [NSString stringWithUTF8String:out];
  panther_free_string(out);
  resolve(res);
}

RCT_REMAP_METHOD(metricsBleu,
                 metricsBleuWithReference:(NSString *)reference
                 candidate:(NSString *)candidate
                 resolver:(RCTPromiseResolveBlock)resolve
                 rejecter:(RCTPromiseRejectBlock)reject)
{
  double score = panther_metrics_bleu([reference UTF8String], [candidate UTF8String]);
  resolve(@(score));
}

RCT_REMAP_METHOD(recordMetric,
                 recordMetricWithName:(NSString *)name
                 resolver:(RCTPromiseResolveBlock)resolve
                 rejecter:(RCTPromiseRejectBlock)reject)
{
  int rc = panther_metrics_record([name UTF8String], 1.0);
  if (rc == 0) resolve(@(rc)); else reject(@"ERR_METRIC", @"record failed", nil);
}

RCT_REMAP_METHOD(listStorageItems,
                 listStorageItemsWithResolver:(RCTPromiseResolveBlock)resolve
                 rejecter:(RCTPromiseRejectBlock)reject)
{
  char* out = panther_storage_list_metrics();
  NSString* res = [NSString stringWithUTF8String:out];
  panther_free_string(out);
  resolve(res);
}

RCT_REMAP_METHOD(validate,
                 validateWithPrompt:(NSString *)prompt
                 resolver:(RCTPromiseResolveBlock)resolve
                 rejecter:(RCTPromiseRejectBlock)reject)
{
  const char* cPrompt = [prompt UTF8String];
  char* out = panther_validation_run_default(cPrompt);
  NSString* res = [NSString stringWithUTF8String:out];
  panther_free_string(out);
  resolve(res);
}

RCT_REMAP_METHOD(validateMultiWithProof,
                 validateWithProofPrompt:(NSString *)prompt
                 providersJson:(NSString *)providersJson
                 resolver:(RCTPromiseResolveBlock)resolve
                 rejecter:(RCTPromiseRejectBlock)reject)
{
  const char* p = [prompt UTF8String];
  const char* j = [providersJson UTF8String];
  char* out = panther_validation_run_multi_with_proof(p, j);
  NSString* res = [NSString stringWithUTF8String:out];
  panther_free_string(out);
  resolve(res);
}

RCT_REMAP_METHOD(validateOpenAI,
                 validateOpenAIPrompt:(NSString *)prompt
                 apiKey:(NSString *)apiKey
                 model:(NSString *)model
                 base:(NSString *)base
                 resolver:(RCTPromiseResolveBlock)resolve
                 rejecter:(RCTPromiseRejectBlock)reject)
{
  const char* p = [prompt UTF8String];
  const char* k = [apiKey UTF8String];
  const char* m = [model UTF8String];
  const char* b = [base UTF8String];
  char* out = panther_validation_run_openai(p, k, m, b);
  NSString* res = [NSString stringWithUTF8String:out];
  panther_free_string(out);
  resolve(res);
}

RCT_REMAP_METHOD(validateOllama,
                 validateOllamaPrompt:(NSString *)prompt
                 base:(NSString *)base
                 model:(NSString *)model
                 resolver:(RCTPromiseResolveBlock)resolve
                 rejecter:(RCTPromiseRejectBlock)reject)
{
  const char* p = [prompt UTF8String];
  const char* b = [base UTF8String];
  const char* m = [model UTF8String];
  char* out = panther_validation_run_ollama(p, b, m);
  NSString* res = [NSString stringWithUTF8String:out];
  panther_free_string(out);
  resolve(res);
}

RCT_REMAP_METHOD(validateMulti,
                 validateMultiPrompt:(NSString *)prompt
                 providersJson:(NSString *)providersJson
                 resolver:(RCTPromiseResolveBlock)resolve
                 rejecter:(RCTPromiseRejectBlock)reject)
{
  const char* p = [prompt UTF8String];
  const char* j = [providersJson UTF8String];
  char* out = panther_validation_run_multi(p, j);
  NSString* res = [NSString stringWithUTF8String:out];
  panther_free_string(out);
  resolve(res);
}

RCT_REMAP_METHOD(validateCustom,
                 validateCustomPrompt:(NSString *)prompt
                 providersJson:(NSString *)providersJson
                 guidelinesJson:(NSString *)guidelinesJson
                 resolver:(RCTPromiseResolveBlock)resolve
                 rejecter:(RCTPromiseRejectBlock)reject)
{
  const char* p = [prompt UTF8String];
  const char* j = [providersJson UTF8String];
  const char* g = [guidelinesJson UTF8String];
  char* out = panther_validation_run_custom(p, j, g);
  NSString* res = [NSString stringWithUTF8String:out];
  panther_free_string(out);
  resolve(res);
}

RCT_REMAP_METHOD(validateCustomWithProof,
                 validateCustomWithProofPrompt:(NSString *)prompt
                 providersJson:(NSString *)providersJson
                 guidelinesJson:(NSString *)guidelinesJson
                 resolver:(RCTPromiseResolveBlock)resolve
                 rejecter:(RCTPromiseRejectBlock)reject)
{
  const char* p = [prompt UTF8String];
  const char* j = [providersJson UTF8String];
  const char* g = [guidelinesJson UTF8String];
  char* out = panther_validation_run_custom_with_proof(p, j, g);
  NSString* res = [NSString stringWithUTF8String:out];
  panther_free_string(out);
  resolve(res);
}

RCT_REMAP_METHOD(getLogs,
                 getLogsWithResolver:(RCTPromiseResolveBlock)resolve
                 rejecter:(RCTPromiseRejectBlock)reject)
{
  char* out = panther_logs_get();
  NSString* res = [NSString stringWithUTF8String:out];
  panther_free_string(out);
  resolve(res);
}

RCT_REMAP_METHOD(tokenCount,
                 tokenCountWithText:(NSString *)text
                 resolver:(RCTPromiseResolveBlock)resolve
                 rejecter:(RCTPromiseRejectBlock)reject)
{
  if (text == nil) { resolve(@(0)); return; }
  int32_t n = panther_token_count([text UTF8String]);
  resolve(@(n));
}

RCT_REMAP_METHOD(calculateCost,
                 calculateCostWithTokensIn:(nonnull NSNumber *)tokensIn
                 tokensOut:(nonnull NSNumber *)tokensOut
                 providerName:(NSString *)providerName
                 costRulesJson:(NSString *)costRulesJson
                 resolver:(RCTPromiseResolveBlock)resolve
                 rejecter:(RCTPromiseRejectBlock)reject)
{
  const int32_t ti = (int32_t)[tokensIn intValue];
  const int32_t to = (int32_t)[tokensOut intValue];
  const char* pn = [providerName UTF8String];
  const char* rj = [costRulesJson UTF8String];
  double cost = panther_calculate_cost(ti, to, pn, rj);
  resolve(@(cost));
}

RCT_REMAP_METHOD(version,
                 versionWithResolver:(RCTPromiseResolveBlock)resolve
                 rejecter:(RCTPromiseRejectBlock)reject)
{
  char* out = panther_version_string();
  NSString* res = [NSString stringWithUTF8String:out];
  panther_free_string(out);
  resolve(res);
}

<<<<<<< HEAD
// --- Guidelines ---
RCT_REMAP_METHOD(guidelinesIngest,
                 guidelinesIngestWithJson:(NSString *)json
                 resolver:(RCTPromiseResolveBlock)resolve
                 rejecter:(RCTPromiseRejectBlock)reject)
{
  int32_t n = panther_guidelines_ingest_json([json UTF8String]);
  resolve(@(n));
}
RCT_REMAP_METHOD(guidelinesScores,
                 guidelinesScoresWithQuery:(NSString *)query
                 topK:(nonnull NSNumber *)topK
                 method:(NSString *)method
                 resolver:(RCTPromiseResolveBlock)resolve
                 rejecter:(RCTPromiseRejectBlock)reject)
{
  char* out = panther_guidelines_similarity([query UTF8String], (int32_t)[topK intValue], [method UTF8String]);
  NSString* res = [NSString stringWithUTF8String:out];
  panther_free_string(out);
  resolve(res);
}
RCT_REMAP_METHOD(guidelinesSave,
                 guidelinesSaveWithName:(NSString *)name
                 json:(NSString *)json
                 resolver:(RCTPromiseResolveBlock)resolve
                 rejecter:(RCTPromiseRejectBlock)reject)
{
  int rc = panther_guidelines_save_json([name UTF8String], [json UTF8String]);
  if (rc == 0) resolve(@(rc)); else reject(@"ERR_GUIDE_SAVE", @"save failed", nil);
}
RCT_REMAP_METHOD(guidelinesLoad,
                 guidelinesLoadWithName:(NSString *)name
                 resolver:(RCTPromiseResolveBlock)resolve
                 rejecter:(RCTPromiseRejectBlock)reject)
{
  int32_t n = panther_guidelines_load([name UTF8String]);
  resolve(@(n));
}
RCT_REMAP_METHOD(guidelinesBuildEmbeddings,
                 guidelinesBuildEmbeddingsWithMethod:(NSString *)method
                 resolver:(RCTPromiseResolveBlock)resolve
                 rejecter:(RCTPromiseRejectBlock)reject)
{
  int32_t n = panther_guidelines_embeddings_build([method UTF8String]);
  resolve(@(n));
}

=======
>>>>>>> origin/main
@end
