-- Your SQL goes here
CREATE TABLE WKTemplateElements (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    workout_template_id UUID NOT NULL REFERENCES WorkoutTemplates(id) ON DELETE CASCADE,
    exercise_id UUID NOT NULL REFERENCES Exercises(id) ON DELETE CASCADE,
    position smallint NOT NULL CHECK (position >= 0),
    reps smallint NOT NULL CHECK (reps >= 0),
    sets smallint NOT NULL CHECK (sets >= 0),
    weight FLOAT4 CHECK (weight IS NULL OR weight >= 0),
    rest smallint NOT NULL CHECK (rest >= 0),
    super_set smallint CHECK (super_set IS NULL OR super_set >= 0)
);
