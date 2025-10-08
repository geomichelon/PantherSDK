#include <jni.h>
#include <string.h>
#include "panther.h" // ensure this header is available in include path

JNIEXPORT jint JNICALL
Java_com_example_panther_PantherBridge_pantherInit(JNIEnv* env, jclass clazz) {
    (void)env; (void)clazz;
    return panther_init();
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

