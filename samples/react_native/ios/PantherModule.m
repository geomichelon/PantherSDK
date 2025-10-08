#import <React/RCTBridgeModule.h>
#include "panther.h"

@interface RCT_EXTERN_MODULE(PantherModule, NSObject)
- (void)init:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
- (void)generate:(NSString *)prompt resolver:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
- (void)metricsBleu:(NSString *)reference candidate:(NSString *)candidate resolver:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
 - (void)recordMetric:(NSString *)name resolver:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
 - (void)listStorageItems:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
 - (void)getLogs:(RCTPromiseResolveBlock)resolve rejecter:(RCTPromiseRejectBlock)reject;
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

RCT_REMAP_METHOD(getLogs,
                 getLogsWithResolver:(RCTPromiseResolveBlock)resolve
                 rejecter:(RCTPromiseRejectBlock)reject)
{
  char* out = panther_logs_get();
  NSString* res = [NSString stringWithUTF8String:out];
  panther_free_string(out);
  resolve(res);
}

@end
