# Data model

See [DATABASE.md](../DATABASE.md) for the live schema.

User-scoped entities: measurements, lab tests, lab results, documents, workouts, medications, symptoms, appointments, profile extras (height, blood type, allergies, emergency info) — collect only what is useful.

Laboratory biomarkers are rows (`biomarker_code`), not columns. Status flags compare a number to that report's range and are not a diagnosis.
