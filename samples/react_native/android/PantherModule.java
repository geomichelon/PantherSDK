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
}
