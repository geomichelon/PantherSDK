// Placeholder header for IDE indexing. Overwritten by cbindgen during builds.
#pragma once
#ifdef __cplusplus
extern "C" {
#endif

int panther_init(void);
char* panther_version_string(void);
char* panther_generate(const char* prompt);
void panther_free_string(char* s);

double panther_metrics_bleu(const char* reference, const char* candidate);
double panther_metrics_accuracy(const char* expected, const char* generated);
double panther_metrics_coherence(const char* text);
double panther_metrics_diversity(const char* samples_json);
double panther_metrics_fluency(const char* text);
double panther_metrics_rouge_l(const char* reference, const char* candidate);
double panther_metrics_fact_coverage(const char* facts_json, const char* candidate);
double panther_metrics_factcheck_adv(const char* facts_json, const char* candidate);
double panther_metrics_plagiarism(const char* corpus_json, const char* candidate);
double panther_metrics_plagiarism_ngram(const char* corpus_json, const char* candidate, int ngram);
int panther_metrics_record(const char* name, double value);

char* panther_bias_detect(const char* samples_json);

int panther_storage_save_metric(const char* name, double value, long long timestamp_ms);
char* panther_storage_get_history(const char* metric);
char* panther_storage_export(const char* format);
char* panther_storage_list_metrics(void);

char* panther_logs_get(void);
char* panther_logs_get_recent(void);

#ifdef __cplusplus
}
#endif
