#include <jni.h>
#include "panther.h"

JNIEXPORT jint JNICALL
Java_com_example_panther_PantherBridge_pantherInit(JNIEnv* env, jclass clazz) {
    (void)env; (void)clazz; return panther_init();
}

JNIEXPORT jstring JNICALL
Java_com_example_panther_PantherBridge_pantherGenerate(JNIEnv* env, jclass clazz, jstring prompt) {
    (void)clazz;
    const char* c_prompt = (*env)->GetStringUTFChars(env, prompt, 0);
    char* out = panther_generate(c_prompt);
    (*env)->ReleaseStringUTFChars(env, prompt, c_prompt);
    jstring result = (*env)->NewStringUTF(env, out);
    panther_free_string(out);
    return result;
}

JNIEXPORT jstring JNICALL
Java_com_example_panther_PantherBridge_biasDetect(JNIEnv* env, jclass clazz, jstring samplesJson) {
    (void)clazz;
    const char* s = (*env)->GetStringUTFChars(env, samplesJson, 0);
    char* out = panther_bias_detect(s);
    (*env)->ReleaseStringUTFChars(env, samplesJson, s);
    jstring result = (*env)->NewStringUTF(env, out);
    panther_free_string(out);
    return result;
}

JNIEXPORT jdouble JNICALL
Java_com_example_panther_PantherBridge_metricsBleu(JNIEnv* env, jclass clazz, jstring reference, jstring candidate) {
    (void)clazz;
    const char* r = (*env)->GetStringUTFChars(env, reference, 0);
    const char* c = (*env)->GetStringUTFChars(env, candidate, 0);
    double score = panther_metrics_bleu(r, c);
    (*env)->ReleaseStringUTFChars(env, reference, r);
    (*env)->ReleaseStringUTFChars(env, candidate, c);
    return score;
}

JNIEXPORT jdouble JNICALL
Java_com_example_panther_PantherBridge_metricsPlagiarism(JNIEnv* env, jclass clazz, jstring corpusJson, jstring candidate) {
    (void)clazz;
    const char* cj = (*env)->GetStringUTFChars(env, corpusJson, 0);
    const char* c = (*env)->GetStringUTFChars(env, candidate, 0);
    double score = panther_metrics_plagiarism(cj, c);
    (*env)->ReleaseStringUTFChars(env, corpusJson, cj);
    (*env)->ReleaseStringUTFChars(env, candidate, c);
    return score;
}

JNIEXPORT jdouble JNICALL
Java_com_example_panther_PantherBridge_metricsPlagiarismNgram(JNIEnv* env, jclass clazz, jstring corpusJson, jstring candidate, jint ngram) {
    (void)clazz;
    const char* cj = (*env)->GetStringUTFChars(env, corpusJson, 0);
    const char* c = (*env)->GetStringUTFChars(env, candidate, 0);
    double score = panther_metrics_plagiarism_ngram(cj, c, (int32_t) (ngram > 0 ? ngram : 3));
    (*env)->ReleaseStringUTFChars(env, corpusJson, cj);
    (*env)->ReleaseStringUTFChars(env, candidate, c);
    return score;
}

JNIEXPORT jstring JNICALL
Java_com_example_panther_PantherBridge_validate(JNIEnv* env, jclass clazz, jstring prompt) {
    (void)clazz;
    const char* c_prompt = (*env)->GetStringUTFChars(env, prompt, 0);
    char* out = panther_validation_run_default(c_prompt);
    (*env)->ReleaseStringUTFChars(env, prompt, c_prompt);
    jstring result = (*env)->NewStringUTF(env, out);
    panther_free_string(out);
    return result;
}

JNIEXPORT jstring JNICALL
Java_com_example_panther_PantherBridge_validateOpenAI(JNIEnv* env, jclass clazz, jstring prompt, jstring apiKey, jstring model, jstring base) {
    (void)clazz;
    const char* p = (*env)->GetStringUTFChars(env, prompt, 0);
    const char* k = (*env)->GetStringUTFChars(env, apiKey, 0);
    const char* m = (*env)->GetStringUTFChars(env, model, 0);
    const char* b = (*env)->GetStringUTFChars(env, base, 0);
    char* out = panther_validation_run_openai(p, k, m, b);
    (*env)->ReleaseStringUTFChars(env, prompt, p);
    (*env)->ReleaseStringUTFChars(env, apiKey, k);
    (*env)->ReleaseStringUTFChars(env, model, m);
    (*env)->ReleaseStringUTFChars(env, base, b);
    jstring result = (*env)->NewStringUTF(env, out);
    panther_free_string(out);
    return result;
}

JNIEXPORT jstring JNICALL
Java_com_example_panther_PantherBridge_validateOllama(JNIEnv* env, jclass clazz, jstring prompt, jstring base, jstring model) {
    (void)clazz;
    const char* p = (*env)->GetStringUTFChars(env, prompt, 0);
    const char* b = (*env)->GetStringUTFChars(env, base, 0);
    const char* m = (*env)->GetStringUTFChars(env, model, 0);
    char* out = panther_validation_run_ollama(p, b, m);
    (*env)->ReleaseStringUTFChars(env, prompt, p);
    (*env)->ReleaseStringUTFChars(env, base, b);
    (*env)->ReleaseStringUTFChars(env, model, m);
    jstring result = (*env)->NewStringUTF(env, out);
    panther_free_string(out);
    return result;
}

JNIEXPORT jstring JNICALL
Java_com_example_panther_PantherBridge_validateCustom(JNIEnv* env, jclass clazz, jstring prompt, jstring providersJson, jstring guidelinesJson) {
    (void)clazz;
    const char* p = (*env)->GetStringUTFChars(env, prompt, 0);
    const char* j = (*env)->GetStringUTFChars(env, providersJson, 0);
    const char* g = (*env)->GetStringUTFChars(env, guidelinesJson, 0);
    char* out = panther_validation_run_custom(p, j, g);
    (*env)->ReleaseStringUTFChars(env, prompt, p);
    (*env)->ReleaseStringUTFChars(env, providersJson, j);
    (*env)->ReleaseStringUTFChars(env, guidelinesJson, g);
    jstring result = (*env)->NewStringUTF(env, out);
    panther_free_string(out);
    return result;
}
JNIEXPORT jstring JNICALL
Java_com_example_panther_PantherBridge_validateMulti(JNIEnv* env, jclass clazz, jstring prompt, jstring providersJson) {
    (void)clazz;
    const char* p = (*env)->GetStringUTFChars(env, prompt, 0);
    const char* j = (*env)->GetStringUTFChars(env, providersJson, 0);
    char* out = panther_validation_run_multi(p, j);
    (*env)->ReleaseStringUTFChars(env, prompt, p);
    (*env)->ReleaseStringUTFChars(env, providersJson, j);
    jstring result = (*env)->NewStringUTF(env, out);
    panther_free_string(out);
    return result;
}

JNIEXPORT jstring JNICALL
Java_com_example_panther_PantherBridge_validateMultiWithProof(JNIEnv* env, jclass clazz, jstring prompt, jstring providersJson) {
    (void)clazz;
    const char* p = (*env)->GetStringUTFChars(env, prompt, 0);
    const char* j = (*env)->GetStringUTFChars(env, providersJson, 0);
    char* out = panther_validation_run_multi_with_proof(p, j);
    (*env)->ReleaseStringUTFChars(env, prompt, p);
    (*env)->ReleaseStringUTFChars(env, providersJson, j);
    jstring result = (*env)->NewStringUTF(env, out);
    panther_free_string(out);
    return result;
}

JNIEXPORT jstring JNICALL
Java_com_example_panther_PantherBridge_validateCustomWithProof(JNIEnv* env, jclass clazz, jstring prompt, jstring providersJson, jstring guidelinesJson) {
    (void)clazz;
    const char* p = (*env)->GetStringUTFChars(env, prompt, 0);
    const char* j = (*env)->GetStringUTFChars(env, providersJson, 0);
    const char* g = (*env)->GetStringUTFChars(env, guidelinesJson, 0);
    char* out = panther_validation_run_custom_with_proof(p, j, g);
    (*env)->ReleaseStringUTFChars(env, prompt, p);
    (*env)->ReleaseStringUTFChars(env, providersJson, j);
    (*env)->ReleaseStringUTFChars(env, guidelinesJson, g);
    jstring result = (*env)->NewStringUTF(env, out);
    panther_free_string(out);
    return result;
}

JNIEXPORT jint JNICALL
Java_com_example_panther_PantherBridge_recordMetric(JNIEnv* env, jclass clazz, jstring name) {
    (void)clazz;
    const char* c = (*env)->GetStringUTFChars(env, name, 0);
    int rc = panther_metrics_record(c, 1.0);
    (*env)->ReleaseStringUTFChars(env, name, c);
    return rc;
}

JNIEXPORT jstring JNICALL
Java_com_example_panther_PantherBridge_listStorageItems(JNIEnv* env, jclass clazz) {
    (void)clazz;
    char* out = panther_storage_list_metrics();
    jstring result = (*env)->NewStringUTF(env, out);
    panther_free_string(out);
    return result;
}

JNIEXPORT jstring JNICALL
Java_com_example_panther_PantherBridge_getLogs(JNIEnv* env, jclass clazz) {
    (void)clazz;
    char* out = panther_logs_get();
    jstring result = (*env)->NewStringUTF(env, out);
    panther_free_string(out);
    return result;
}

JNIEXPORT jstring JNICALL
Java_com_example_panther_PantherBridge_version(JNIEnv* env, jclass clazz) {
    (void)clazz;
    char* out = panther_version_string();
    jstring result = (*env)->NewStringUTF(env, out);
    panther_free_string(out);
    return result;
}

JNIEXPORT jint JNICALL
Java_com_example_panther_PantherBridge_tokenCount(JNIEnv* env, jclass clazz, jstring text) {
    (void)clazz;
    const char* t = (*env)->GetStringUTFChars(env, text, 0);
    int32_t n = panther_token_count(t);
    (*env)->ReleaseStringUTFChars(env, text, t);
    return (jint)n;
}

JNIEXPORT jdouble JNICALL
Java_com_example_panther_PantherBridge_calculateCost(JNIEnv* env, jclass clazz, jint tokensIn, jint tokensOut, jstring providerName, jstring costRulesJson) {
    (void)clazz;
    const char* p = (*env)->GetStringUTFChars(env, providerName, 0);
    const char* r = (*env)->GetStringUTFChars(env, costRulesJson, 0);
    double cost = panther_calculate_cost((int32_t)tokensIn, (int32_t)tokensOut, p, r);
    (*env)->ReleaseStringUTFChars(env, providerName, p);
    (*env)->ReleaseStringUTFChars(env, costRulesJson, r);
    return cost;
}

// --- Guidelines similarity JNI wrappers ---
JNIEXPORT jint JNICALL
Java_com_example_panther_PantherBridge_guidelinesIngest(JNIEnv* env, jclass clazz, jstring json) {
    (void)clazz;
    const char* j = (*env)->GetStringUTFChars(env, json, 0);
    int32_t n = panther_guidelines_ingest_json(j);
    (*env)->ReleaseStringUTFChars(env, json, j);
    return (jint)n;
}

JNIEXPORT jstring JNICALL
Java_com_example_panther_PantherBridge_guidelinesScores(JNIEnv* env, jclass clazz, jstring query, jint topK, jstring method) {
    (void)clazz;
    const char* q = (*env)->GetStringUTFChars(env, query, 0);
    const char* m = (*env)->GetStringUTFChars(env, method, 0);
    char* out = panther_guidelines_similarity(q, (int32_t)topK, m);
    (*env)->ReleaseStringUTFChars(env, query, q);
    (*env)->ReleaseStringUTFChars(env, method, m);
    jstring result = (*env)->NewStringUTF(env, out);
    panther_free_string(out);
    return result;
}

JNIEXPORT jint JNICALL
Java_com_example_panther_PantherBridge_guidelinesSave(JNIEnv* env, jclass clazz, jstring name, jstring json) {
    (void)clazz;
    const char* n = (*env)->GetStringUTFChars(env, name, 0);
    const char* j = (*env)->GetStringUTFChars(env, json, 0);
    int rc = panther_guidelines_save_json(n, j);
    (*env)->ReleaseStringUTFChars(env, name, n);
    (*env)->ReleaseStringUTFChars(env, json, j);
    return (jint)rc;
}

JNIEXPORT jint JNICALL
Java_com_example_panther_PantherBridge_guidelinesLoad(JNIEnv* env, jclass clazz, jstring name) {
    (void)clazz;
    const char* n = (*env)->GetStringUTFChars(env, name, 0);
    int32_t cnt = panther_guidelines_load(n);
    (*env)->ReleaseStringUTFChars(env, name, n);
    return (jint)cnt;
}

JNIEXPORT jint JNICALL
Java_com_example_panther_PantherBridge_guidelinesBuildEmbeddings(JNIEnv* env, jclass clazz, jstring method) {
    (void)clazz;
    const char* m = (*env)->GetStringUTFChars(env, method, 0);
    int32_t rc = panther_guidelines_embeddings_build(m);
    (*env)->ReleaseStringUTFChars(env, method, m);
    return (jint)rc;
}
