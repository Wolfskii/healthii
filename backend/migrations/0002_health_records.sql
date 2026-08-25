-- User-scoped health records. Every table carries user_id (or a parent that does).

CREATE TABLE profiles (
    user_id UUID PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
    height_cm DOUBLE PRECISION,
    blood_type TEXT,
    allergies TEXT,
    medical_history TEXT,
    emergency_name TEXT,
    emergency_phone TEXT,
    unit_system TEXT NOT NULL DEFAULT 'metric',
    weight_unit TEXT NOT NULL DEFAULT 'kg',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT profiles_unit_system_check CHECK (unit_system IN ('metric', 'imperial')),
    CONSTRAINT profiles_weight_unit_check CHECK (weight_unit IN ('kg', 'lb'))
);

INSERT INTO profiles (user_id)
SELECT id FROM users
ON CONFLICT (user_id) DO NOTHING;

CREATE TABLE measurements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    type TEXT NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    unit TEXT NOT NULL,
    measured_at TIMESTAMPTZ NOT NULL,
    source TEXT NOT NULL DEFAULT 'manual',
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX measurements_user_type_time_idx
    ON measurements (user_id, type, measured_at DESC);

CREATE TABLE lab_tests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    tested_on DATE NOT NULL,
    laboratory TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX lab_tests_user_date_idx ON lab_tests (user_id, tested_on DESC);

CREATE TABLE lab_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lab_test_id UUID NOT NULL REFERENCES lab_tests (id) ON DELETE CASCADE,
    biomarker_code TEXT NOT NULL,
    name TEXT,
    value DOUBLE PRECISION,
    value_text TEXT,
    unit TEXT,
    reference_low DOUBLE PRECISION,
    reference_high DOUBLE PRECISION,
    reference_text TEXT,
    status TEXT NOT NULL DEFAULT 'unknown',
    CONSTRAINT lab_results_status_check
        CHECK (status IN ('low', 'normal', 'high', 'critical', 'unknown'))
);

CREATE INDEX lab_results_test_idx ON lab_results (lab_test_id);
CREATE INDEX lab_results_code_idx ON lab_results (biomarker_code);

CREATE TABLE biomarker_definitions (
    code TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    default_unit TEXT,
    category TEXT
);

INSERT INTO biomarker_definitions (code, name, default_unit, category) VALUES
    ('hemoglobin', 'Hemoglobin', 'g/L', 'blood'),
    ('hematocrit', 'Hematocrit', '%', 'blood'),
    ('rbc', 'RBC', '10^12/L', 'blood'),
    ('wbc', 'WBC', '10^9/L', 'blood'),
    ('platelets', 'Platelets', '10^9/L', 'blood'),
    ('ferritin', 'Ferritin', 'ug/L', 'iron'),
    ('iron', 'Iron', 'umol/L', 'iron'),
    ('vitamin_d', 'Vitamin D', 'nmol/L', 'vitamins'),
    ('vitamin_b12', 'Vitamin B12', 'pmol/L', 'vitamins'),
    ('tsh', 'TSH', 'mIU/L', 'thyroid'),
    ('free_t4', 'Free T4', 'pmol/L', 'thyroid'),
    ('glucose', 'Glucose', 'mmol/L', 'metabolic'),
    ('hba1c', 'HbA1c', 'mmol/mol', 'metabolic'),
    ('total_cholesterol', 'Total Cholesterol', 'mmol/L', 'lipids'),
    ('ldl', 'LDL', 'mmol/L', 'lipids'),
    ('hdl', 'HDL', 'mmol/L', 'lipids'),
    ('triglycerides', 'Triglycerides', 'mmol/L', 'lipids'),
    ('alt', 'ALT', 'U/L', 'liver'),
    ('ast', 'AST', 'U/L', 'liver'),
    ('creatinine', 'Creatinine', 'umol/L', 'kidney'),
    ('egfr', 'eGFR', 'mL/min/1.73m2', 'kidney'),
    ('crp', 'CRP', 'mg/L', 'inflammation')
ON CONFLICT (code) DO NOTHING;

CREATE TABLE documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    size BIGINT NOT NULL,
    storage_key TEXT NOT NULL UNIQUE,
    checksum TEXT,
    document_type TEXT NOT NULL DEFAULT 'other',
    title TEXT,
    description TEXT,
    document_date DATE,
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT documents_type_check CHECK (document_type IN (
        'doctor_report', 'blood_test', 'prescription', 'imaging',
        'vaccination', 'referral', 'discharge_summary', 'other'
    ))
);

CREATE INDEX documents_user_idx ON documents (user_id, uploaded_at DESC);

CREATE TABLE workouts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    workout_type TEXT NOT NULL DEFAULT 'other',
    started_at TIMESTAMPTZ NOT NULL,
    duration_seconds INTEGER,
    distance DOUBLE PRECISION,
    distance_unit TEXT,
    calories INTEGER,
    notes TEXT,
    exercises JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT workouts_type_check CHECK (workout_type IN (
        'strength', 'running', 'cycling', 'walking', 'swimming',
        'sports', 'mobility', 'other'
    ))
);

CREATE INDEX workouts_user_idx ON workouts (user_id, started_at DESC);

CREATE TABLE medications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    kind TEXT NOT NULL DEFAULT 'medication',
    dosage TEXT,
    schedule TEXT,
    started_on DATE,
    ended_on DATE,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT medications_kind_check CHECK (kind IN ('medication', 'supplement'))
);

CREATE INDEX medications_user_idx ON medications (user_id, name);

CREATE TABLE symptoms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    severity SMALLINT,
    noted_at TIMESTAMPTZ NOT NULL,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT symptoms_severity_check CHECK (severity IS NULL OR (severity >= 1 AND severity <= 10))
);

CREATE INDEX symptoms_user_idx ON symptoms (user_id, noted_at DESC);

CREATE TABLE appointments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    location TEXT,
    provider TEXT,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX appointments_user_idx ON appointments (user_id, starts_at);
