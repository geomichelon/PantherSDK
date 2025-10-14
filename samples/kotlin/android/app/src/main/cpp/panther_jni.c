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
