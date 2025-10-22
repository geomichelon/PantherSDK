package com.example.panther;

import com.facebook.react.bridge.Promise;
import com.facebook.react.bridge.ReactApplicationContext;
import com.facebook.react.bridge.ReactContextBaseJavaModule;
import com.facebook.react.bridge.ReactMethod;

public class PantherModule extends ReactContextBaseJavaModule {
    static {
        System.loadLibrary("panther_ffi");
    }

    public PantherModule(ReactApplicationContext context) {
        super(context);
    }

    @Override
    public String getName() { return "PantherModule"; }

    private static native int pantherInit();
    private static native String pantherGenerate(String prompt);
    private static native double metricsBleu(String reference, String candidate);
    private static native int recordMetric(String name);
    private static native String listStorageItems();
    private static native String getLogs();
    private static native String validate(String prompt);
    private static native String validateMulti(String prompt, String providersJson);
    private static native String version();
    private static native String validateMultiWithProof(String prompt, String providersJson);
    private static native String validateCustomWithProof(String prompt, String providersJson, String guidelinesJson);
    private static native String validateOpenAI(String prompt, String apiKey, String model, String base);
    private static native String validateOllama(String prompt, String base, String model);
    private static native String validateCustom(String prompt, String providersJson, String guidelinesJson);
    private static native int tokenCount(String text);
    private static native double calculateCost(int tokensIn, int tokensOut, String providerName, String costRulesJson);
<<<<<<< HEAD
    // Guidelines
    private static native int pantherGuidelinesIngest(String json);
    private static native String pantherGuidelinesScores(String query, int topK, String method);
    private static native int pantherGuidelinesSave(String name, String json);
    private static native int pantherGuidelinesLoad(String name);
    private static native int pantherGuidelinesEmbeddingsBuild(String method);
=======
>>>>>>> origin/main

    @ReactMethod
    public void init(Promise promise) {
        int rc = pantherInit();
        if (rc == 0) promise.resolve(rc); else promise.reject("ERR_INIT", "panther_init failed");
    }

    @ReactMethod
    public void generate(String prompt, Promise promise) {
        promise.resolve(pantherGenerate(prompt));
    }

    @ReactMethod
    public void metricsBleu(String reference, String candidate, Promise promise) {
        promise.resolve(metricsBleu(reference, candidate));
    }

    @ReactMethod
    public void recordMetric(String name, Promise promise) {
        int rc = recordMetric(name);
        if (rc == 0) promise.resolve(rc); else promise.reject("ERR_METRIC", "record failed");
    }

    @ReactMethod
    public void listStorageItems(Promise promise) {
        promise.resolve(listStorageItems());
    }

    @ReactMethod
    public void getLogs(Promise promise) {
        promise.resolve(getLogs());
    }

    @ReactMethod
    public void validate(String prompt, Promise promise) {
        promise.resolve(validate(prompt));
    }

    @ReactMethod
    public void validateMulti(String prompt, String providersJson, Promise promise) {
        promise.resolve(validateMulti(prompt, providersJson));
    }

    @ReactMethod
    public void version(Promise promise) {
        promise.resolve(version());
    }

    @ReactMethod
    public void validateMultiWithProof(String prompt, String providersJson, Promise promise) {
        promise.resolve(validateMultiWithProof(prompt, providersJson));
    }

    @ReactMethod
    public void validateCustomWithProof(String prompt, String providersJson, String guidelinesJson, Promise promise) {
        promise.resolve(validateCustomWithProof(prompt, providersJson, guidelinesJson));
    }

    @ReactMethod
    public void validateOpenAI(String prompt, String apiKey, String model, String base, Promise promise) {
        promise.resolve(validateOpenAI(prompt, apiKey, model, base));
    }

    @ReactMethod
    public void validateOllama(String prompt, String base, String model, Promise promise) {
        promise.resolve(validateOllama(prompt, base, model));
    }

    @ReactMethod
    public void validateCustom(String prompt, String providersJson, String guidelinesJson, Promise promise) {
        promise.resolve(validateCustom(prompt, providersJson, guidelinesJson));
    }

    @ReactMethod
    public void tokenCount(String text, Promise promise) {
        promise.resolve(tokenCount(text));
    }

    @ReactMethod
    public void calculateCost(int tokensIn, int tokensOut, String providerName, String costRulesJson, Promise promise) {
        promise.resolve(calculateCost(tokensIn, tokensOut, providerName, costRulesJson));
    }
<<<<<<< HEAD

    // --- Guidelines ---
    @ReactMethod
    public void guidelinesIngest(String json, Promise promise) { promise.resolve(pantherGuidelinesIngest(json)); }

    @ReactMethod
    public void guidelinesScores(String query, int topK, String method, Promise promise) { promise.resolve(pantherGuidelinesScores(query, topK, method)); }

    @ReactMethod
    public void guidelinesSave(String name, String json, Promise promise) {
        int rc = pantherGuidelinesSave(name, json);
        if (rc == 0) promise.resolve(rc); else promise.reject("ERR_GUIDE_SAVE", "save failed");
    }

    @ReactMethod
    public void guidelinesLoad(String name, Promise promise) { promise.resolve(pantherGuidelinesLoad(name)); }

    @ReactMethod
    public void guidelinesBuildEmbeddings(String method, Promise promise) { promise.resolve(pantherGuidelinesEmbeddingsBuild(method)); }
=======
>>>>>>> origin/main
}
