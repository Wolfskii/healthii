# Laboratory results

Tables (planned): `lab_tests`, `lab_results`, `biomarker_definitions`, `reference_ranges`.

A result may include numeric value, text value, unit, reference low/high/text and status (`low|normal|high|critical|unknown`).

Statuses compare a number to the range on that report. They are **not** a diagnosis. UI copy must say so.

Do not hardcode a biomarker list in the schema. Seed common definitions (hemoglobin, TSH, vitamin D, lipids, …) as rows.
