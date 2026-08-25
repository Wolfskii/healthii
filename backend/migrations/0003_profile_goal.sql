-- Optional weight goal. Original unit is stored; never overwrite a measurement with a converted value.

ALTER TABLE profiles
    ADD COLUMN goal_weight DOUBLE PRECISION,
    ADD COLUMN goal_weight_unit TEXT;

ALTER TABLE profiles
    ADD CONSTRAINT profiles_goal_weight_unit_check
    CHECK (goal_weight_unit IS NULL OR goal_weight_unit IN ('kg', 'lb'));
